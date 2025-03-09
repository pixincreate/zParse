//! zParse: A zero-dependency parser for JSON and TOML formats
//!
//! This crate provides functionality to:
//! - Parse JSON and TOML documents
//! - Convert between JSON and TOML formats
//! - Pretty print parsed documents
//! - Handle errors with detailed context
//!
//! # Examples
//!
//! ## Parsing a file
//! ```no_run
//! use zparse::{parse_file, error::Result};
//!
//! fn example() -> Result<()> {
//!     let value = parse_file("config.json")?;
//!     println!("Parsed value: {}", value);
//!     Ok(())
//! }
//! ```
//!
//! ## Parsing from string
//! ```
//! use zparse::{parse_json, error::Result, ValueExt};
//!
//! fn example() -> Result<()> {
//!     let json_str = r#"{"name": "test", "value": 42}"#;
//!     let value = parse_json(json_str)?;
//!     println!("Parsed name: {}", value.get_string("name").unwrap_or_default());
//!     println!("Parsed value: {}", value.get_number("value").unwrap_or_default());
//!     Ok(())
//! }
//! ```
//!
//! ## Converting between formats
//! ```no_run
//! use zparse::{parse_json, converter::Converter, error::Result};
//!
//! fn example() -> Result<()> {
//!     let json_str = r#"{"name": "test", "value": 42}"#;
//!     let json_value = parse_json(json_str)?;
//!
//!     // Convert JSON to TOML
//!     let toml_value = Converter::json_to_toml(&json_value)?;
//!
//!     // Format as TOML string
//!     let toml_str = zparse::format_toml(&toml_value)?;
//!     println!("TOML output:\n{}", toml_str);
//!     Ok(())
//! }
//! ```

use tracing::{debug, info, instrument, warn};

pub mod converter;
pub mod enums;
pub mod error;
pub mod formatter;
pub mod parser;
#[doc(hidden)]
pub mod test_utils;
pub mod utils;

// Re-exports for convenient access
pub use error::{IOError, ParseError, ParseErrorKind, Result, SemanticError};
pub use parser::value::Value;
pub use utils::{format_json, format_toml, parse_json, parse_toml};

/// Parse a file by automatically detecting its format from extension
///
/// # Arguments
/// * `path` - Path to the file to parse
///
/// # Returns
/// * `Result<Value>` - The parsed value or an error
///
/// # Format Detection
/// - Files ending with `.json` are parsed as JSON
/// - Files ending with `.toml` are parsed as TOML
/// - Other extensions will return an error
///
/// # Example
/// ```no_run
/// use zparse::{parse_file, error::Result};
///
/// fn example() -> Result<()> {
///     let value = parse_file("config.json")?;
///     println!("Parsed value: {}", value);
///     Ok(())
/// }
/// ```
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

/// Extended capabilities for Value objects
pub trait ValueExt {
    /// Gets a string value from a map by key
    ///
    /// # Returns
    /// Some(string) if the key exists and value is a string, None otherwise
    fn get_string(&self, key: &str) -> Option<String>;

    /// Gets a number value from a map by key
    ///
    /// # Returns
    /// Some(number) if the key exists and value is a number, None otherwise
    fn get_number(&self, key: &str) -> Option<f64>;

    /// Gets a boolean value from a map by key
    ///
    /// # Returns
    /// Some(bool) if the key exists and value is a boolean, None otherwise
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Gets a map value from a map by key
    ///
    /// # Returns
    /// Some(map) if the key exists and value is a map, None otherwise
    fn get_map(&self, key: &str) -> Option<&Value>;

    /// Gets an array value from a map by key
    ///
    /// # Returns
    /// Some(array) if the key exists and value is an array, None otherwise
    fn get_array(&self, key: &str) -> Option<&Value>;
}

impl ValueExt for Value {
    fn get_string(&self, key: &str) -> Option<String> {
        match self {
            Self::Map(map) => map.get(key).and_then(|v| {
                if let Self::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    fn get_number(&self, key: &str) -> Option<f64> {
        match self {
            Self::Map(map) => map.get(key).and_then(|v| {
                if let Self::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        match self {
            Self::Map(map) => map.get(key).and_then(|v| {
                if let Self::Boolean(b) = v {
                    Some(*b)
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    fn get_map(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Map(map) => {
                map.get(key)
                    .and_then(|v| if let Self::Map(_) = v { Some(v) } else { None })
            }
            _ => None,
        }
    }

    fn get_array(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Map(map) => map.get(key).and_then(|v| {
                if let Self::Array(_) = v {
                    Some(v)
                } else {
                    None
                }
            }),
            _ => None,
        }
    }
}
