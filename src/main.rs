// 1. Fix main.rs - Replace Vec with heapless::Vec and use a fixed capacity
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
use heapless::Vec; // Import heapless Vec for no_std environment

#[macro_use]
extern crate simple_kernel;

use simple_kernel::commands;

entry_point!(kernel_main);

const COMMANDS: &str = include_str!("../commands.txt");

// Import x86_64 instructions (hlt)
use x86_64::instructions::hlt;

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("           Cog OS      ");
    println!("----------------------------");
    
    println!("Running commands from commands.txt:\n");

    for line in COMMANDS.lines() {
        // Split command line into command + args by whitespace
        // Use heapless::Vec with a reasonable capacity
        let mut parts: Vec<&str, 16> = Vec::new();
        for part in line.split_whitespace() {
            if parts.push(part).is_err() {
                println!("Too many arguments in command line");
                break;
            }
        }
        
        if parts.is_empty() {
            continue;
        }
        let cmd = parts[0];
        let args = &parts[1..];
        commands::run_command(cmd, args);
    }

    println!("\nDone executing commands.");

    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        hlt();
    }
}
