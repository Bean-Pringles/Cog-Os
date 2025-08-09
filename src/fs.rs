#![allow(dead_code)]
use core::fmt;
use spin::Mutex;

pub type Result<T> = core::result::Result<T, FsError>;

#[derive(Debug, Clone, Copy)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NoSpace,
    NameTooLong,
    TooManyBlocks,
    InvalidArg,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::NotFound => write!(f, "Not found"),
            FsError::AlreadyExists => write!(f, "Already exists"),
            FsError::NoSpace => write!(f, "No space left"),
            FsError::NameTooLong => write!(f, "Name too long"),
            FsError::TooManyBlocks => write!(f, "File too large"),
            FsError::InvalidArg => write!(f, "Invalid argument"),
        }
    }
}

pub const MAX_FILES: usize = 16;
pub const FILENAME_LEN: usize = 16;
const BLOCK_SIZE: usize = 256; // bytes per block
const BLOCKS: usize = 64;      // total blocks -> 16 KiB total storage
const MAX_BLOCKS_PER_FILE: usize = 16; // max file size = 4096 bytes

#[derive(Clone, Copy)]
struct FileEntry {
    used: bool,
    name: [u8; FILENAME_LEN],
    size: usize, // number of bytes stored
    blocks: [u8; MAX_BLOCKS_PER_FILE], // block indices; 0xFF = unused
    blocks_used: u8,
}

impl Default for FileEntry {
    fn default() -> Self {
        FileEntry {
            used: false,
            name: [0; FILENAME_LEN],
            size: 0,
            blocks: [0xFF; MAX_BLOCKS_PER_FILE],
            blocks_used: 0,
        }
    }
}

pub struct SimpleFs {
    entries: [FileEntry; MAX_FILES],
    block_map: [bool; BLOCKS],       // true if block is used
    data: [[u8; BLOCK_SIZE]; BLOCKS] // block data storage
}

impl SimpleFs {
    pub const fn new() -> Self {
        const FE: FileEntry = FileEntry {
            used: false,
            name: [0; FILENAME_LEN],
            size: 0,
            blocks: [0xFF; MAX_BLOCKS_PER_FILE],
            blocks_used: 0,
        };
        SimpleFs {
            entries: [FE; MAX_FILES],
            block_map: [false; BLOCKS],
            data: [[0u8; BLOCK_SIZE]; BLOCKS],
        }
    }

    fn find_entry(&self, name: &str) -> Option<usize> {
        for (i, e) in self.entries.iter().enumerate() {
            if e.used {
                if let Ok(s) = core::str::from_utf8(&e.name) {
                    let s_trim = s.trim_end_matches(char::from(0));
                    if s_trim == name {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    fn find_free_entry(&mut self) -> Option<usize> {
        for (i, e) in self.entries.iter().enumerate() {
            if !e.used {
                return Some(i);
            }
        }
        None
    }

    fn allocate_blocks(&mut self, blocks_needed: usize) -> Result<[u8; MAX_BLOCKS_PER_FILE]> {
        if blocks_needed > MAX_BLOCKS_PER_FILE {
            return Err(FsError::TooManyBlocks);
        }
        let mut out = [0xFFu8; MAX_BLOCKS_PER_FILE];
        let mut found = 0usize;
        for i in 0..BLOCKS {
            if !self.block_map[i] {
                self.block_map[i] = true;
                out[found] = i as u8;
                found += 1;
                if found == blocks_needed {
                    break;
                }
            }
        }
        if found < blocks_needed {
            // Undo partial allocation
            for j in 0..found {
                let idx = out[j] as usize;
                self.block_map[idx] = false;
                out[j] = 0xFF;
            }
            return Err(FsError::NoSpace);
        }
        Ok(out)
    }

    fn free_blocks(&mut self, blocks: &[u8], blocks_used: usize) {
        for i in 0..blocks_used {
            let b = blocks[i];
            if b != 0xFF {
                self.block_map[b as usize] = false;
                for byte in self.data[b as usize].iter_mut() {
                    *byte = 0;
                }
            }
        }
    }

    pub fn create(&mut self, name: &str) -> Result<()> {
        if name.is_empty() || name.len() > FILENAME_LEN {
            return Err(FsError::NameTooLong);
        }
        if self.find_entry(name).is_some() {
            return Err(FsError::AlreadyExists);
        }
        let idx = self.find_free_entry().ok_or(FsError::NoSpace)?;
        let mut e = FileEntry::default();
        e.used = true;
        e.size = 0;
        e.blocks_used = 0;
        let nb = name.as_bytes();
        for i in 0..nb.len() {
            e.name[i] = nb[i];
        }
        self.entries[idx] = e;
        Ok(())
    }

    pub fn delete(&mut self, name: &str) -> Result<()> {
        let idx = self.find_entry(name).ok_or(FsError::NotFound)?;

        let (blocks, blocks_used);
        {
            let e = &mut self.entries[idx];
            blocks = e.blocks;
            blocks_used = e.blocks_used as usize;
            *e = FileEntry::default();
        } // mutable borrow ends here

        self.free_blocks(&blocks, blocks_used);
        Ok(())
    }

    pub fn write(&mut self, name: &str, data: &[u8]) -> Result<()> {
        if name.is_empty() || name.len() > FILENAME_LEN {
            return Err(FsError::NameTooLong);
        }

        // Free old blocks if file exists
        if let Some(idx) = self.find_entry(name) {
            let (old_blocks, old_blocks_used);
            {
                let e = &mut self.entries[idx];
                old_blocks = e.blocks;
                old_blocks_used = e.blocks_used as usize;
                e.blocks = [0xFF; MAX_BLOCKS_PER_FILE];
                e.blocks_used = 0;
                e.size = 0;
            } // drop borrow here

            self.free_blocks(&old_blocks, old_blocks_used);
        } else {
            // Create new entry
            let idx = self.find_free_entry().ok_or(FsError::NoSpace)?;
            let mut e = FileEntry::default();
            e.used = true;
            let nb = name.as_bytes();
            for i in 0..nb.len() {
                e.name[i] = nb[i];
            }
            self.entries[idx] = e;
        }

        let blocks_needed = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        if blocks_needed > MAX_BLOCKS_PER_FILE {
            return Err(FsError::TooManyBlocks);
        }

        // Allocate blocks before borrowing entry mutably
        let allocated = self.allocate_blocks(blocks_needed)?;

        let idx = self.find_entry(name).unwrap();
        let e = &mut self.entries[idx];

        for i in 0..blocks_needed {
            let block_idx = allocated[i] as usize;
            let start = i * BLOCK_SIZE;
            let end = core::cmp::min(start + BLOCK_SIZE, data.len());
            let slice = &data[start..end];

            // Clear block before write
            for byte in self.data[block_idx].iter_mut() {
                *byte = 0;
            }

            // Copy data into block
            for (j, &b) in slice.iter().enumerate() {
                self.data[block_idx][j] = b;
            }

            e.blocks[i] = allocated[i];
        }
        e.blocks_used = blocks_needed as u8;
        e.size = data.len();

        Ok(())
    }

    pub fn read_into(&self, name: &str, buf: &mut [u8]) -> Result<usize> {
        let idx = self.find_entry(name).ok_or(FsError::NotFound)?;
        let e = &self.entries[idx];
        let to_copy = core::cmp::min(buf.len(), e.size);
        let mut written = 0usize;
        let mut remaining = to_copy;
        for bi in 0..(e.blocks_used as usize) {
            if remaining == 0 {
                break;
            }
            let block_idx = e.blocks[bi] as usize;
            let take = core::cmp::min(remaining, BLOCK_SIZE);
            let src = &self.data[block_idx][..take];
            let dst = &mut buf[written..written + take];
            for i in 0..take {
                dst[i] = src[i];
            }
            written += take;
            remaining -= take;
        }
        Ok(written)
    }

    // List files using owned heapless strings
    pub fn list_files(&self, out: &mut [(Option<heapless::String<FILENAME_LEN>>, usize); MAX_FILES]) {
        use heapless::String;
        let mut j = 0usize;
        for e in self.entries.iter() {
            if e.used {
                let mut name_str: String<FILENAME_LEN> = String::new();
                if let Ok(s) = core::str::from_utf8(&e.name) {
                    let trimmed = s.trim_end_matches(char::from(0));
                    let _ = name_str.push_str(trimmed);
                }
                out[j] = (Some(name_str), e.size);
                j += 1;
            }
        }
        // Fill remaining slots with None
        for k in j..MAX_FILES {
            out[k] = (None, 0);
        }
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref FS: Mutex<SimpleFs> = Mutex::new(SimpleFs::new());
}