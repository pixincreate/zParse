use std::fs;

use crate::error::{IOError, ParseError, ParseErrorKind, Result};

pub struct TestData {
    pub small_json: String,
    pub medium_json: String,
    pub large_json: String,
    pub small_toml: String,
    pub medium_toml: String,
    pub large_toml: String,
}

impl TestData {
    pub fn load() -> Result<Self> {
        Ok(Self {
            small_json: read_test_file("tests/input/small.json")?,
            medium_json: read_test_file("tests/input/file.json")?,
            large_json: read_test_file("tests/input/large.json")?,
            small_toml: read_test_file("tests/input/small.toml")?,
            medium_toml: read_test_file("tests/input/file.toml")?,
            large_toml: read_test_file("tests/input/large.toml")?,
        })
    }
}

pub fn read_test_file(path: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|e| {
        ParseError::new(ParseErrorKind::IO(IOError::ReadError(format!(
            "Failed to read {}: {}",
            path, e
        ))))
    })
}
