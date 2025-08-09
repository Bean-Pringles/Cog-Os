// src/host_block_device.rs
#[cfg(feature = "host")]
use std::fs::{File, OpenOptions};
#[cfg(feature = "host")]
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(feature = "host")]
use std::path::Path;

#[cfg(feature = "host")]
pub struct HostBlockDevice {
    file: File,
    blocks: u64,
}

#[cfg(feature = "host")]
impl HostBlockDevice {
    pub fn open<P: AsRef<Path>>(path: P, blocks: u64) -> std::io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(HostBlockDevice { file: f, blocks })
    }

    pub fn read_block(&mut self, block_num: u64, buf: &mut [u8]) -> std::io::Result<()> {
        let pos = block_num * buf.len() as u64;
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.read_exact(buf)?;
        Ok(())
    }

    pub fn write_block(&mut self, block_num: u64, buf: &[u8]) -> std::io::Result<()> {
        let pos = block_num * buf.len() as u64;
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.write_all(buf)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn total_blocks(&self) -> u64 {
        self.blocks
    }
}