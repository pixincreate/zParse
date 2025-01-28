//! Error handling types for the parser
//!
//! This module provides custom error types that give detailed information about
//! parsing failures, including line and column information where available.

use std::{error::Error, fmt};

/// Represents a parsing error with optional location information
#[derive(Debug)]
pub struct ParseError {
    /// The specific kind of error that occurred
    kind: ParseErrorKind,
    /// Location in the input where the error occurred (if available)
    location: Option<Location>,
    /// Optional source error that caused this error
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

/// Specific types of parsing errors that can occur
#[derive(Debug)]
pub enum ParseErrorKind {
    /// Found an invalid token in the input
    InvalidToken(String),
    /// Found a valid token in an unexpected position
    UnexpectedToken(String),
    /// Reached end of input unexpectedly
    UnexpectedEOF,
    /// Invalid number format
    InvalidNumber(String),
    /// Invalid string format
    InvalidString(String),
    /// Invalid boolean value
    InvalidBoolean,
    /// Invalid date and time value
    InvalidDateTime,
    /// Invalid escape sequence in string
    InvalidEscape(char),
    /// Invalid Unicode escape sequence in string
    InvalidUnicode,
    /// Invalid array value
    InvalidValue(String),
    /// Invalid key in table
    InvalidKey(String),
    /// Invalid table definition
    NestedTableError,
    /// Invalid file or extension
    IoError(String),
    /// Error reading file
    UnknownFormat,
    /// Maximum nesting depth exceeded
    MaxDepthExceeded,
    /// Maximum input size exceeded
    MaxSizeExceeded,
    /// Maximum string length exceeded
    MaxStringLengthExceeded,
    /// Maximum number of object entries exceeded
    MaxObjectEntriesExceeded,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind) -> Self {
        Self {
            kind,
            location: None,
            source: None,
        }
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
        write!(f, "{:?}", self.kind)?;
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
