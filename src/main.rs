#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};

#[macro_use]
mod vga_buffer;
mod commands;

entry_point!(kernel_main);

// Embed file at compile time
const COMMANDS: &str = include_str!("../commands.txt");

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("           Cog OS      ");
    println!("----------------------------");
    println!("Running commands from commands.txt:\n");

    for line in COMMANDS.lines() {
        commands::run_command(line);
    }

    println!("\nDone executing commands.");

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}