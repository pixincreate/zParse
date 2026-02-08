//! zParse - High-performance JSON/TOML/YAML/XML parser
//!
//! # Quick Start
//!
//! ```
//! use zparse::from_str;
//!
//! let value = from_str(r#"{"name": "John", "age": 30}"#).unwrap();
//! assert_eq!(value.as_object().unwrap()["name"].as_string().unwrap(), "John");
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
pub use json::{Config, Event, Parser};

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

/// Convenience re-exports
pub use json::{Config as JsonConfig, Parser as JsonParser};
pub use lexer::json::JsonLexer;
