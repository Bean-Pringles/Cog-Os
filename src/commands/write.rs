// src/commands/write.rs
use crate::FILESYSTEM;
use heapless::String;

pub fn run(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: write <filename> <data>");
        return;
    }
    
    let filename = args[0];
    
    // Manually join the arguments with spaces
    let mut data: String<512> = String::new(); // 512 byte capacity
    for (i, arg) in args[1..].iter().enumerate() {
        if i > 0 {
            if data.push(' ').is_err() {
                crate::println!("Data too long");
                return;
            }
        }
        if data.push_str(arg).is_err() {
            crate::println!("Data too long");
            return;
        }
    }
    
    let mut fs = FILESYSTEM.lock();
    
    match fs.write(filename, data.as_bytes()) {
        Ok(()) => crate::println!("Written {} bytes to file: {}", data.len(), filename),
        Err(e) => crate::println!("Error writing to file '{}': {}", filename, e),
    }
}