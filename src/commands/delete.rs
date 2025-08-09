// src/commands/rm.rs
use crate::println;
use crate::fs::FS;

pub fn run(args: &str) {
    let name = args.trim();
    if name.is_empty() {
        println!("Usage: delete <filename>");
        return;
    }
    let mut fs = FS.lock();
    match fs.delete(name) {
        Ok(()) => println!("Deleted '{}'", name),
        Err(e) => println!("Error deleting '{}': {}", name, e),
    }
}