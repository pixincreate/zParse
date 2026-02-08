//! zParse - High-performance JSON/TOML/YAML/XML parser
//!
//! # Quick Start
//!
//! ```
//! use zparse::from_str;
//! # fn main() -> Result<(), zparse::Error> {
//! let value = from_str(r#"{"name": "John", "age": 30}"#)?;
//! let name = value
//!     .as_object()
//!     .and_then(|obj| obj.get("name"))
//!     .and_then(|v| v.as_string())
//!     .unwrap_or_default();
//! assert_eq!(name, "John");
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod error;
pub use error::{Error, ErrorKind, Pos, Result, Span};

pub mod input;
pub use input::Input;

pub mod lexer;
pub use lexer::{Token, TokenKind};

pub mod value;
pub use value::{Array, Object, Value};

pub mod json;
pub mod toml;
pub use json::{Config, Event, Parser};
pub use toml::{Config as TomlConfig, Parser as TomlParser};

/// Parse JSON from string
pub fn from_str(s: &str) -> Result<Value> {
    let input = Input::from_str(s);
    let mut parser = Parser::new(input.as_bytes());
    parser.parse_value()
}

/// Parse JSON from bytes
pub fn from_bytes(bytes: &[u8]) -> Result<Value> {
    let input = Input::from_bytes(bytes);
    let mut parser = Parser::new(input.as_bytes());
    parser.parse_value()
}

/// Parse with custom configuration
pub fn from_str_with_config(s: &str, config: Config) -> Result<Value> {
    let input = Input::from_str(s);
    let mut parser = Parser::with_config(input.as_bytes(), config);
    parser.parse_value()
}

/// Parse TOML from string
pub fn from_toml_str(s: &str) -> Result<Value> {
    let mut parser = TomlParser::new(s.as_bytes());
    parser.parse()
}

/// Parse TOML from bytes
pub fn from_toml_bytes(bytes: &[u8]) -> Result<Value> {
    let mut parser = TomlParser::new(bytes);
    parser.parse()
}

/// Parse TOML with custom configuration
pub fn from_toml_str_with_config(s: &str, config: TomlConfig) -> Result<Value> {
    let mut parser = TomlParser::with_config(s.as_bytes(), config);
    parser.parse()
}

/// Convenience re-exports
pub use json::{Config as JsonConfig, Parser as JsonParser};
pub use lexer::json::JsonLexer;
