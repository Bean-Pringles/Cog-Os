use crate::FILESYSTEM;

pub fn run(args: &[&str]) {
    if args.len() != 1 {
        crate::println!("Usage: create <filename>");
        return;
    }
    
    let filename = args[0];
    let mut fs = FILESYSTEM.lock();
    
    match fs.create(filename) {
        Ok(()) => crate::println!("Created file: {}", filename),
        Err(e) => crate::println!("Error creating file '{}': {}", filename, e),
    }
}