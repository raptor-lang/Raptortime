use std::io;
use std::io::Read;
use std::fs::File;
use std::error::Error;
use std::path::Path;

pub fn should_open<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if path.is_dir() {
        return false;
    }
    let file = path; // We know it's a file now
    
    let mut ext_iter = ::ACCEPTABLE_EXTENSIONS.into_iter();
    let extension = file.extension().unwrap();

    return ext_iter.any(|&e| e == extension);
}

// Wrapper for open_file
pub fn try_open_file<P: AsRef<Path>>(file_path: P, debug: bool) -> Vec<u8> {
    // Create a Path and a Display to the desired file
    let file_display = file_path.as_ref().display();
    
    // Call open_file and handle Result
    match open_file(&file_path) {
        Err(why) => 
            panic!("Couldn't open file {}: {}", file_display,
                why.description()),
        Ok(data) => {
            if debug {
                debug!("Read {} bytes from file: {}.", data.len(), file_display);
            }
            data
        },
    }
}

fn open_file<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
    // try! to open the file
    let mut file = try!(File::open(file_path));

    // create the buffer
    let mut file_buffer: Vec<u8> = Vec::new();

    // try! to read the data
    try!(file.read_to_end(&mut file_buffer));

    // no panic! issued so we're good
    return Ok(file_buffer);
}
