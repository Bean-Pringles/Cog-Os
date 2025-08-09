use crate::FILESYSTEM;

pub fn run(_args: &[&str]) {
    let fs = FILESYSTEM.lock();
    crate::println!("Files in filesystem:");
    fs.list();
}