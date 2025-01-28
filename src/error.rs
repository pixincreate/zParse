//! Error handling types for the parser
//!
//! This module provides custom error types that give detailed information about
//! parsing failures, including line and column information where available.

use std::{error::Error, fmt};

/// Main error type for parsing operations
#[derive(Debug)]
pub struct ParseError {
    /// The specific kind of error
    kind: ParseErrorKind,
    /// Location where the error occurred
    location: Option<Location>,
    /// Source error that caused this error
    source: Option<Box<dyn Error>>,
}

/// Represents a location in the input text
#[derive(Debug)]
pub struct Location {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// Specific categories of parsing errors
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    // Lexical errors
    /// Found an invalid token in the input
    InvalidToken(String),
    /// Found a valid token in an unexpected position
    UnexpectedToken(String),
    /// Reached end of input unexpectedly
    UnexpectedEOF,

    // Number parsing errors
    /// Found an invalid number format
    InvalidNumber(String),
    /// Number is too large to be represented
    NumberOverflow,
    /// Number is too small to be represented
    NumberUnderflow,

    // String parsing errors
    /// Invalid string format
    InvalidString(String),
    /// Invalid escape sequence in a string
    InvalidEscape(char),
    /// Invalid Unicode escape sequence
    InvalidUnicode,
    /// Unterminated string
    UnterminatedString,

    // Structural errors
    /// Invalid object key format
    InvalidKey(String),
    /// Duplicate object key
    DuplicateKey(String),
    /// Value passed to a function is not a valid type
    InvalidValue(String),
    /// Nested table error (TOML specific)
    NestedTableError,
    /// Circular reference detected in the input
    CircularReference,

    // Security errors
    /// Exceeded maximum depth of nesting
    MaxDepthExceeded,
    /// Exceeded maximum input size
    MaxSizeExceeded,
    /// Exceeded maximum string length
    MaxStringLengthExceeded,
    /// Exceeded maximum number of object entries
    MaxObjectEntriesExceeded,

    // IO errors
    /// Error reading from a file
    IoError(String),
    /// Error parsing a file due to unknwon format
    UnknownFormat,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind) -> Self {
        Self {
            kind,
            location: None,
            source: None,
        }
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.location = Some(Location { line, column });
        self
    }

    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::InvalidToken(t) => write!(f, "Invalid token: {}", t),
            ParseErrorKind::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
            ParseErrorKind::UnexpectedEOF => write!(f, "Unexpected end of input"),
            ParseErrorKind::InvalidNumber(n) => write!(f, "Invalid number format: {}", n),
            ParseErrorKind::NumberOverflow => write!(f, "Number too large"),
            ParseErrorKind::NumberUnderflow => write!(f, "Number too small"),
            ParseErrorKind::InvalidString(s) => write!(f, "Invalid string: {}", s),
            ParseErrorKind::InvalidEscape(c) => write!(f, "Invalid escape sequence: {}", c),
            ParseErrorKind::InvalidUnicode => write!(f, "Invalid Unicode escape sequence"),
            ParseErrorKind::UnterminatedString => write!(f, "Unterminated string"),
            ParseErrorKind::InvalidKey(k) => write!(f, "Invalid key: {}", k),
            ParseErrorKind::DuplicateKey(k) => write!(f, "Duplicate key: {}", k),
            ParseErrorKind::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ParseErrorKind::NestedTableError => write!(f, "Invalid nested table structure"),
            ParseErrorKind::CircularReference => write!(f, "Circular reference detected"),
            ParseErrorKind::MaxDepthExceeded => write!(f, "Maximum nesting depth exceeded"),
            ParseErrorKind::MaxSizeExceeded => write!(f, "Maximum input size exceeded"),
            ParseErrorKind::MaxStringLengthExceeded => write!(f, "Maximum string length exceeded"),
            ParseErrorKind::MaxObjectEntriesExceeded => {
                write!(f, "Maximum number of object entries exceeded")
            }
            ParseErrorKind::IoError(e) => write!(f, "IO error: {}", e),
            ParseErrorKind::UnknownFormat => write!(f, "Unknown file format"),
        }?;

        if let Some(loc) = &self.location {
            write!(f, " at line {}, column {}", loc.line, loc.column)?;
        }
        Ok(())
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(Box::as_ref)
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;
