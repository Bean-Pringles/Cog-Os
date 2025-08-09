use crate::fs::{FS, FILENAME_LEN, MAX_FILES};
use heapless::String;

pub fn run(_args: &str) {
    let fs = FS.lock();

    // Initialize output array with None entries (fixes Copy trait error)
    let mut out: [(Option<String<FILENAME_LEN>>, usize); MAX_FILES] = core::array::from_fn(|_| (None, 0));

    // Fill out with file names and sizes from FS
    fs.list_files(&mut out);

    // Print the file list
    for (opt_name, size) in out.iter() {
        if let Some(name) = opt_name {
            println!("{} - {} bytes", name, size);
        }
    }
}