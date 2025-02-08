//! zParse: A zero-dependency parser for JSON and TOML formats
//!
//! This crate provides functionality to:
//! - Parse JSON and TOML documents
//! - Convert between JSON and TOML formats
//! - Pretty print parsed documents
//! - Handle errors with detailed context
//!
//! # Examples
//! ```
//! use zparse::{parse_file, Result};
//!
//! fn example() -> Result<()> {
//!     let value = parse_file("config.json")?;
//!     println!("Parsed value: {}", value);
//!     Ok(())
//! }
//! ```

use tracing::{debug, info, instrument, warn};

pub mod common;
pub mod converter;
pub mod enums;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod test_utils;
pub mod utils;

// Re-exports
pub use converter::Converter;
pub use error::{IOError, ParseError, ParseErrorKind, Result, SemanticError};
pub use parser::{json::JsonParser, toml::TomlParser, value::Value};
use utils::{parse_json, parse_toml};

pub use common::value_compare::values_equal;

#[instrument]
pub fn parse_file(path: &str) -> Result<Value> {
    debug!("Starting to parse file: {}", path);

    let content = std::fs::read_to_string(path)
        .map_err(|e| ParseError::new(ParseErrorKind::IO(IOError::ReadError(e.to_string()))))?;

    info!("File read successfully, determining format");

    let result = if path.ends_with(".json") {
        parse_json(&content)
    } else if path.ends_with(".toml") {
        parse_toml(&content)
    } else {
        warn!("Unknown file extension");
        Err(ParseError::new(ParseErrorKind::Semantic(
            SemanticError::UnknownFormat,
        )))
    };

    debug!("Parsing completed");
    result
}
