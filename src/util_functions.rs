use crate::enums::{FileError, FileType};
use std::fs;
use std::io;

pub fn read_file(filename: &str) -> Result<String, FileError> {
    fs::read_to_string(filename).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FileError::NotFound,
        _ => FileError::Io(e),
    })
}

pub fn validate_file(file_type: FileType, filename: &str) {
    if let Ok(metadata) = fs::metadata(filename) {
        if metadata.is_file() {
            let file_extension = filename.split('.').last();

            match file_type {
                FileType::Json => {
                    if file_extension != Some("json") {
                        eprintln!("Invalid file type: {}", filename);
                        std::process::exit(1);
                    }
                }
                FileType::Toml => {
                    if file_extension != Some("toml") {
                        eprintln!("Invalid file type: {}", filename);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
