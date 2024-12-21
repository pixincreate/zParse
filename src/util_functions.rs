use crate::enums::FileError;
use std::fs;
use std::io;

pub fn read_file(filename: &str) -> Result<String, FileError> {
    fs::read_to_string(filename).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FileError::NotFound,
        _ => FileError::Io(e),
    })
}
