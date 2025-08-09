// src/fs.rs
use core::fmt;
use core::result::Result::{Ok, Err};
use core::option::Option::{Some, None};

// Import the println macro (assuming it's exported from your crate root)
use crate::println;

pub const BLOCK_SIZE: usize = 512;
pub const MAX_FILES: usize = 32;
pub const FILENAME_LEN: usize = 16;
pub const MAX_FILE_BLOCKS: usize = 128;

#[derive(Debug, Clone, Copy)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NoSpace,
    NameTooLong,
    FileTooLarge,
    IoError,
    InvalidArg,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FsError::NotFound => write!(f, "Not found"),
            FsError::AlreadyExists => write!(f, "Already exists"),
            FsError::NoSpace => write!(f, "No space left"),
            FsError::NameTooLong => write!(f, "Name too long"),
            FsError::FileTooLarge => write!(f, "File too large"),
            FsError::IoError => write!(f, "IO error"),
            FsError::InvalidArg => write!(f, "Invalid argument"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; FILENAME_LEN],
    pub size: u32,
    pub blocks: u32,
    pub block_list: [u32; MAX_FILE_BLOCKS],
}

impl Default for DirEntry {
    fn default() -> DirEntry {
        DirEntry {
            name: [0u8; FILENAME_LEN],
            size: 0,
            blocks: 0,
            block_list: [0; MAX_FILE_BLOCKS],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Superblock {
    pub magic: u32,
    pub block_size: u32,
    pub max_files: u32,
    pub total_blocks: u32,
    pub free_block_bitmap: [u8; 64], // bitmap for 512 blocks
}

impl Default for Superblock {
    fn default() -> Superblock {
        Superblock {
            magic: 0xF5F0F0F5,
            block_size: BLOCK_SIZE as u32,
            max_files: MAX_FILES as u32,
            total_blocks: 512,
            free_block_bitmap: [0xFF; 64], // all blocks free at start
        }
    }
}

pub struct FS {
    pub superblock: Superblock,
    pub dir: [DirEntry; MAX_FILES],
    pub data: [[u8; BLOCK_SIZE]; 512], // In-memory data blocks
}

impl FS {
    pub fn new() -> FS {
        FS {
            superblock: Superblock::default(),
            dir: [DirEntry::default(); MAX_FILES],
            data: [[0u8; BLOCK_SIZE]; 512],
        }
    }

    fn alloc_block(&mut self) -> Result<u32, FsError> {
        for (byte_idx, byte) in self.superblock.free_block_bitmap.iter_mut().enumerate() {
            if *byte != 0 {
                for bit in 0..8 {
                    if (*byte & (1 << bit)) != 0 {
                        *byte &= !(1 << bit);
                        return Ok((byte_idx * 8 + bit) as u32);
                    }
                }
            }
        }
        Err(FsError::NoSpace)
    }

    fn free_block(&mut self, block_num: u32) {
        let byte_idx = (block_num / 8) as usize;
        let bit = block_num % 8;
        self.superblock.free_block_bitmap[byte_idx] |= 1 << bit;
    }

    pub fn create(&mut self, name: &str) -> Result<(), FsError> {
        if name.len() == 0 || name.len() > FILENAME_LEN {
            return Err(FsError::NameTooLong);
        }
        for entry in self.dir.iter() {
            if entry.name[0] != 0 {
                let entry_name = core::str::from_utf8(&entry.name)
                    .unwrap_or_default()
                    .trim_end_matches(char::from(0));
                if entry_name == name {
                    return Err(FsError::AlreadyExists);
                }
            }
        }
        for entry in self.dir.iter_mut() {
            if entry.name[0] == 0 {
                for (i, b) in name.bytes().enumerate() {
                    entry.name[i] = b;
                }
                entry.size = 0;
                entry.blocks = 0;
                entry.block_list = [0; MAX_FILE_BLOCKS];
                return Ok(());
            }
        }
        Err(FsError::NoSpace)
    }

    pub fn delete(&mut self, name: &str) -> Result<(), FsError> {
        // Find the entry first
        let entry_index = self.dir.iter().position(|entry| {
            if entry.name[0] != 0 {
                let entry_name = core::str::from_utf8(&entry.name)
                    .unwrap_or_default()
                    .trim_end_matches(char::from(0));
                entry_name == name
            } else {
                false
            }
        });

        if let Some(idx) = entry_index {
            // Collect blocks to free first (avoid borrowing conflicts)
            let blocks_to_free: [u32; MAX_FILE_BLOCKS] = self.dir[idx].block_list;
            let blocks_count = self.dir[idx].blocks as usize;
            
            // Free the blocks
            for block in &blocks_to_free[..blocks_count] {
                self.free_block(*block);
            }
            
            // Clear the entry
            self.dir[idx].name = [0; FILENAME_LEN];
            self.dir[idx].size = 0;
            self.dir[idx].blocks = 0;
            self.dir[idx].block_list = [0; MAX_FILE_BLOCKS];
            
            Ok(())
        } else {
            Err(FsError::NotFound)
        }
    }

    pub fn write(&mut self, name: &str, data: &[u8]) -> Result<(), FsError> {
        // Find the entry first
        let entry_index = self.dir.iter().position(|entry| {
            if entry.name[0] != 0 {
                let entry_name = core::str::from_utf8(&entry.name)
                    .unwrap_or_default()
                    .trim_end_matches(char::from(0));
                entry_name == name
            } else {
                false
            }
        });

        let entry_idx = match entry_index {
            Some(idx) => idx,
            None => return Err(FsError::NotFound),
        };

        let blocks_needed = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        if blocks_needed > MAX_FILE_BLOCKS {
            return Err(FsError::FileTooLarge);
        }

        // Collect old blocks to free first
        let old_blocks: [u32; MAX_FILE_BLOCKS] = self.dir[entry_idx].block_list;
        let old_blocks_count = self.dir[entry_idx].blocks as usize;
        
        // Free old blocks
        for block in &old_blocks[..old_blocks_count] {
            self.free_block(*block);
        }

        // Reset blocks count
        self.dir[entry_idx].blocks = 0;

        // Allocate new blocks and write data
        for i in 0..blocks_needed {
            let block_num = self.alloc_block()?;
            self.dir[entry_idx].block_list[i] = block_num;
            self.dir[entry_idx].blocks += 1;
            let start = i * BLOCK_SIZE;
            let end = usize::min(start + BLOCK_SIZE, data.len());
            let block_data = &data[start..end];
            self.data[block_num as usize][..block_data.len()].copy_from_slice(block_data);
            if block_data.len() < BLOCK_SIZE {
                for b in &mut self.data[block_num as usize][block_data.len()..] {
                    *b = 0;
                }
            }
        }
        self.dir[entry_idx].size = data.len() as u32;
        Ok(())
    }

    pub fn read(&self, name: &str, buf: &mut [u8]) -> Result<usize, FsError> {
        for entry in self.dir.iter() {
            if entry.name[0] != 0 {
                let entry_name = core::str::from_utf8(&entry.name)
                    .unwrap_or_default()
                    .trim_end_matches(char::from(0));
                if entry_name == name {
                    let size = usize::min(buf.len(), entry.size as usize);
                    let mut copied = 0;
                    for i in 0..entry.blocks as usize {
                        let block_num = entry.block_list[i] as usize;
                        let block_start = i * BLOCK_SIZE;
                        if block_start >= size {
                            break;
                        }
                        let copy_len = usize::min(BLOCK_SIZE, size - block_start);
                        buf[copied..copied+copy_len]
                            .copy_from_slice(&self.data[block_num][..copy_len]);
                        copied += copy_len;
                    }
                    return Ok(copied);
                }
            }
        }
        Err(FsError::NotFound)
    }

    pub fn list(&self) {
        for entry in self.dir.iter() {
            if entry.name[0] != 0 {
                let entry_name = core::str::from_utf8(&entry.name)
                    .unwrap_or_default()
                    .trim_end_matches(char::from(0));
                println!("{} - {} bytes", entry_name, entry.size);
            }
        }
    }
}