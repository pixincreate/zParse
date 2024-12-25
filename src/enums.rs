use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use std::io;
use strum::EnumIter;
use thiserror::Error;
use toml::de::Error as TomlError;

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, PartialEq, ValueEnum)]
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
    #[error("Not a file")]
    NotFile,
    #[error("File extension is invalid or missing")]
    InvalidExtension,
    #[error(
        "Wrong file extension. Expected: {expected}, found: {found}. Supported types: {supported}"
    )]
    WrongExtension {
        expected: String,
        found: String,
        supported: String,
    },
}

#[derive(Error, Debug)]
pub enum ContentError {
    #[error("Error parsing JSON: {0}")]
    InvalidJson(#[from] SerdeJsonError),
    #[error("Error parsing TOML: {0}")]
    InvalidToml(#[from] TomlError),
}

#[derive(Error, Debug)]
pub enum SkillIssue {
    #[error("Usage: cargo run -- --file-name <filename>")]
    WrongCommand,
}
