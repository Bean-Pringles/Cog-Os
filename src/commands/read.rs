use crate::FILESYSTEM;

pub fn run(args: &[&str]) {
    if args.len() != 1 {
        crate::println!("Usage: read <filename>");
        return;
    }
    
    let filename = args[0];
    let fs = FILESYSTEM.lock();
    
    let mut buffer = [0u8; 4096];
    match fs.read(filename, &mut buffer) {
        Ok(bytes_read) => {
            crate::println!("Read {} bytes from file '{}':", bytes_read, filename);
            if let Ok(content) = core::str::from_utf8(&buffer[..bytes_read]) {
                crate::println!("{}", content);
            } else {
                crate::println!("File contains binary data");
            }
        }
        Err(e) => crate::println!("Error reading file '{}': {}", filename, e),
    }
}