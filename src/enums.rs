use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Json,
    Toml,
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found")]
    NotFound,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
