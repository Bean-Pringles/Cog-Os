// main.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod vga_buffer;
mod keyboard;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // Initialize keyboard
    keyboard::init();
    
    println!("           Cog OS      ");
    println!("                       ");
    println!("     12 Hours of Work");
    println!("The World Should say Hello Back");
    println!("");
    println!("Type keys and see them appear below:");
    
    loop {
        // Use the non-blocking version that handles polling internally
        if let Some(c) = keyboard::try_read_char() {
            if c == '\n' {
                println!(); // Handle newlines properly
            } else {
                print!("{}", c);
            }
        }
        
        // Give CPU a break
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