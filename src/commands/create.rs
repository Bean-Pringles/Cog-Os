// src/commands/touch.rs
use crate::println;
use crate::fs::FS;

pub fn run(args: &str) {
    let name = args.trim();
    if name.is_empty() {
        println!("Usage: create <filename>");
        return;
    }
    let mut fs = FS.lock();
    match fs.create(name) {
        Ok(()) => println!("Created '{}'", name),
        Err(e) => println!("Error creating '{}': {}", name, e),
    }
}