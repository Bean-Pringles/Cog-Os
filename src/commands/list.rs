use crate::fs::{FS, FILENAME_LEN, MAX_FILES};
use heapless::String;

pub fn run(_args: &str) {
    let fs = FS.lock();

    // Use curly braces around const generic param
    let mut out: [(Option<String<{FILENAME_LEN}>>, usize); MAX_FILES] = core::array::from_fn(|_| (None, 0));

    fs.list_files(&mut out);

    let mut any_file = false;
    for (opt_name, size) in out.iter() {
        if let Some(name) = opt_name {
            println!("{} - {} bytes", name, size);
            any_file = true;
        }
    }

    if !any_file {
        println!("No files found.");
    }
}