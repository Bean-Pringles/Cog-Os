// src/commands/write_cmd.rs
use crate::println;
use crate::fs::FS;

pub fn run(args: &str) {
    // usage: write <filename> <text...>
    let mut parts = args.splitn(2, ' ');
    let name = parts.next().unwrap_or("").trim();
    let data = parts.next().unwrap_or("");

    if name.is_empty() {
        println!("Usage: write <filename> <text...>");
        return;
    }

    let mut fs = FS.lock();
    match fs.write(name, data.as_bytes()) {
        Ok(()) => println!("Wrote {} bytes to '{}'", data.len(), name),
        Err(e) => println!("Write error: {}", e),
    }
}