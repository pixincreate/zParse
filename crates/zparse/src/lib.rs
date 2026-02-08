//! zParse - High-performance JSON/TOML/YAML/XML parser

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
