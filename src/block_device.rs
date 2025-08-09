// src/block_device.rs
pub trait BlockDevice {
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<(), ()>;
    fn write_block(&mut self, block_num: u64, buf: &[u8]) -> Result<(), ()>;
    fn total_blocks(&self) -> u64;
}