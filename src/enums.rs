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
    Io(io::Error),

    #[error("Not a file")]
    NotFile,

    #[error("Invalid file extension")]
    InvalidExtension,

    #[error("Wrong extension: expected {expected}, found {found}. Supported: {supported}")]
    WrongExtension {
        expected: String,
        found: String,
        supported: String,
    },

    #[error("Invalid output file type: {found}. Supported types: {supported}")]
    InvalidOutputType { found: String, supported: String },

    #[error("Invalid input file type: {found}. Supported types: {supported}")]
    InvalidInputType { found: String, supported: String },
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
