// src/commands/read_cmd.rs
use crate::println;
use crate::fs::FS;

pub fn run(args: &str) {
    let name = args.trim();
    if name.is_empty() {
        println!("Usage: read <filename>");
        return;
    }

    let mut buf = [0u8; 4096]; // must match MAX_BLOCKS_PER_FILE * BLOCK_SIZE
    let fs = FS.lock();
    match fs.read_into(name, &mut buf) {
        Ok(n) => {
            if n == 0 {
                println!("(empty)");
                return;
            }
            // Convert to str safely
            if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                println!("{}", s);
            } else {
                // fallback: print bytes as hex or as lossy
                for &b in &buf[..n] {
                    if b >= 0x20 && b <= 0x7e {
                        // printable
                        crate::print!("{}", b as char);
                    } else {
                        crate::print!("\\x{:02x}", b);
                    }
                }
                crate::print!("\n");
            }
        }
        Err(e) => println!("Error reading '{}': {}", name, e),
    }
}