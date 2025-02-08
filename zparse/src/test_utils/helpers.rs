use crate::error::{IOError, ParseError, ParseErrorKind, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn tmp_file_path(name: &str) -> PathBuf {
    let mut dir = env::temp_dir();
    dir.push("zparse_tests");
    let _ = fs::create_dir_all(&dir);
    dir.push(name);
    dir
}

pub fn create_temp_file(content: &str) -> Result<PathBuf> {
    let path = tmp_file_path("temp_test_file");
    fs::write(&path, content)
        .map_err(|e| ParseError::new(ParseErrorKind::IO(IOError::WriteError(e.to_string()))))?;
    Ok(path)
}

pub fn cleanup_temp_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}
