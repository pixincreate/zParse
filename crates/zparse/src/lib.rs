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
pub use value::{Array, Object, TomlDatetime, Value};

pub mod convert;
pub use convert::{convert, Format};

pub mod json;
pub mod toml;
pub mod xml;
pub mod yaml;
pub use json::{Config, Event, Parser};
pub use toml::{Config as TomlConfig, Parser as TomlParser};
pub use xml::{
    Content as XmlContent, Document as XmlDocument, Element as XmlElement, Parser as XmlParser,
};
pub use yaml::{Config as YamlConfig, Parser as YamlParser};

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

/// Parse YAML from string
pub fn from_yaml_str(s: &str) -> Result<Value> {
    let mut parser = YamlParser::new(s.as_bytes());
    parser.parse()
}

/// Parse YAML from bytes
pub fn from_yaml_bytes(bytes: &[u8]) -> Result<Value> {
    let mut parser = YamlParser::new(bytes);
    parser.parse()
}

/// Parse YAML with custom configuration
pub fn from_yaml_str_with_config(s: &str, config: YamlConfig) -> Result<Value> {
    let mut parser = YamlParser::with_config(s.as_bytes(), config);
    parser.parse()
}

/// Parse XML from string
pub fn from_xml_str(s: &str) -> Result<XmlDocument> {
    let mut parser = XmlParser::new(s.as_bytes());
    parser.parse()
}

/// Parse XML from bytes
pub fn from_xml_bytes(bytes: &[u8]) -> Result<XmlDocument> {
    let mut parser = XmlParser::new(bytes);
    parser.parse()
}

/// Convenience re-exports
pub use json::{Config as JsonConfig, Parser as JsonParser};
pub use lexer::json::JsonLexer;
pub use lexer::yaml::YamlLexer;
