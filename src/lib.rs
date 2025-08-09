// src/lib.rs
#![cfg_attr(not(feature = "host"), no_std)]

// Shared modules
pub mod block_device;
pub mod fs;
pub mod commands;

// Kernel-only display (VGA)
#[cfg(not(feature = "host"))]
#[macro_use]
pub mod vga_buffer;

// Host-only module
#[cfg(feature = "host")]
pub mod host_block_device;

// Only define print/println macros for kernel mode
#[cfg(not(feature = "host"))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[cfg(not(feature = "host"))]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// For host mode, use std println
#[cfg(feature = "host")]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        std::println!($($arg)*)
    };
}

// Global filesystem instance
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FILESYSTEM: Mutex<fs::FS> = Mutex::new(fs::FS::new());
}