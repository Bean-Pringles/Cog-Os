use crate::FILESYSTEM;

pub fn run(args: &[&str]) {
    if args.len() != 1 {
        crate::println!("Usage: delete <filename>");
        return;
    }
    
    let filename = args[0];
    let mut fs = FILESYSTEM.lock();
    
    match fs.delete(filename) {
        Ok(()) => crate::println!("Deleted file: {}", filename),
        Err(e) => crate::println!("Error deleting file '{}': {}", filename, e),
    }
}