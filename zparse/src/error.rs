//! Error handling types for the parser
//!
//! This module provides custom error types that give detailed information about
//! parsing failures, including line and column information where available.

use std::{error::Error, fmt};

use crate::parser::lexer::Lexer;

/// Main error type for parsing operations
#[derive(Debug)]
pub struct ParseError {
    /// The specific kind of error
    kind: ParseErrorKind,
    /// Location where the error occurred
    location: Option<Location>,
    /// Source error that caused this error
    source: Option<Box<dyn Error>>,
    /// Additional context for the error
    context: Option<String>,
}

/// Represents a location in the input text
#[derive(Debug)]
pub struct Location {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// Top-level error categories
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    Conversion(ConversionError),
    Format(FormatError),
    IO(IOError),
    Lexical(LexicalError),
    Security(SecurityError),
    Semantic(SemanticError),
    Syntax(SyntaxError),
}

/// Conversion errors
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Unsupported value type
    UnsupportedValue(String),
}

/// Format errors
#[derive(Debug, Clone)]
pub enum FormatError {
    /// Invalid indentation in the input
    InvalidIndentation(String),
    /// Invalid value in the input
    InvalidValue(String),
    /// Error serializing a value
    SerializationFailed(String),
}

/// Lexical analysis errors
#[derive(Debug, Clone)]
pub enum LexicalError {
    /// Invalid escape sequence in a string
    InvalidEscape(char),
    /// Found an invalid number format
    InvalidNumber(String),
    /// Invalid string format
    InvalidString(String),
    /// Found an invalid token in the input
    InvalidToken(String),
    /// Invalid Unicode escape sequence
    InvalidUnicode,
    /// Number is too large to be represented
    NumberOverflow,
    /// Number is too small to be represented
    NumberUnderflow,
    /// Found a valid token in an unexpected position
    UnexpectedToken(String),
    /// Reached end of input unexpectedly
    UnexpectedEOF,
    /// Unterminated string
    UnterminatedString,
}

/// Syntax parsing errors
#[derive(Debug, Clone)]
pub enum SyntaxError {
    /// Duplicate object key
    DuplicateKey(String),
    /// Invalid object key format
    InvalidKey(String),
    /// Value passed to a function is not a valid type
    InvalidValue(String),
    /// Missing colon after key
    MissingColon,
    /// Missing comma between elements
    MissingComma,
    /// Trailing comma in an array or object
    TrailingComma,
    /// Found an unexpected character in the input
    UnexpectedCharacter(char),
}

/// Semantic validation errors
#[derive(Debug, Clone)]
pub enum SemanticError {
    /// Circular reference detected in the input
    CircularReference,
    /// Invalid document format
    InvalidFormat,
    /// Nested table error (TOML specific)
    NestedTableError,
    /// Type mismatch error
    TypeMismatch(String),
    /// Error parsing a file due to unknown format
    UnknownFormat,
}

/// Security-related errors
#[derive(Debug, Clone)]
pub enum SecurityError {
    /// Exceeded maximum depth of nesting
    MaxDepthExceeded,
    /// Exceeded maximum number of object entries
    MaxObjectEntriesExceeded,
    /// Exceeded maximum input size
    MaxSizeExceeded,
    /// Exceeded maximum string length
    MaxStringLengthExceeded,
}

/// IO operation errors
#[derive(Debug, Clone)]
pub enum IOError {
    /// File not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Error reading from a file
    ReadError(String),
    /// Error writing to a file
    WriteError(String),
}

impl ParseError {
    pub fn new(kind: ParseErrorKind) -> Self {
        Self {
            kind,
            location: None,
            source: None,
            context: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.location = Some(Location { line, column });
        self
    }

    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
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
        // Start with base error description
        let base_error = match &self.kind {
            ParseErrorKind::Conversion(err) => err.to_string(),
            ParseErrorKind::Format(err) => err.to_string(),
            ParseErrorKind::IO(err) => err.to_string(),
            ParseErrorKind::Lexical(err) => err.to_string(),
            ParseErrorKind::Security(err) => err.to_string(),
            ParseErrorKind::Semantic(err) => err.to_string(),
            ParseErrorKind::Syntax(err) => err.to_string(),
        };

        // Format with location if available
        if let Some(loc) = &self.location {
            write!(
                f,
                "at line {}, column {}: {}",
                loc.line, loc.column, base_error
            )?;
        } else {
            write!(f, "Error: {}", base_error)?;
        }

        // Add context if available
        if let Some(ctx) = &self.context {
            write!(f, "\nContext: {}", ctx)?;
        }

        // Add source if available
        if let Some(source) = &self.source {
            write!(f, "\nCaused by: {}", source)?;
        }

        Ok(())
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedValue(v) => write!(f, "Unsupported value: '{}'", v),
        }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIndentation(i) => write!(f, "Invalid indentation: '{}'", i),
            Self::InvalidValue(v) => write!(f, "Invalid value: '{}'", v),
            Self::SerializationFailed(s) => write!(f, "Serialization failed: '{}'", s),
        }
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEscape(c) => write!(f, "Invalid escape sequence '\\{}'", c),
            Self::InvalidNumber(n) => write!(f, "Invalid number format: '{}'", n),
            Self::InvalidString(s) => write!(f, "Invalid string format: '{}'", s),
            Self::InvalidToken(t) => write!(f, "Unexpected token: {}", t),
            Self::InvalidUnicode => write!(f, "Invalid Unicode escape sequence"),
            Self::NumberOverflow => write!(f, "Number is too large to represent"),
            Self::NumberUnderflow => write!(f, "Number is too small to represent"),
            Self::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
            Self::UnexpectedEOF => write!(f, "Unexpected end of file"),
            Self::UnterminatedString => write!(f, "Unterminated string literal"),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey(k) => write!(f, "Duplicate key '{}' in object", k),
            Self::InvalidKey(k) => write!(f, "Invalid key format: '{}'", k),
            Self::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            Self::MissingColon => write!(f, "Missing colon after object key"),
            Self::MissingComma => write!(f, "Missing comma between elements"),
            Self::TrailingComma => write!(f, "Trailing comma is not allowed"),
            Self::UnexpectedCharacter(c) => write!(f, "Unexpected character '{}'", c),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircularReference => write!(f, "Circular reference detected"),
            Self::InvalidFormat => write!(f, "Invalid document format"),
            Self::NestedTableError => write!(f, "Invalid nested table structure"),
            Self::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            Self::UnknownFormat => write!(f, "Unknown file format"),
        }
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxDepthExceeded => write!(f, "Maximum nesting depth exceeded"),
            Self::MaxObjectEntriesExceeded => {
                write!(f, "Maximum number of object entries exceeded")
            }
            Self::MaxSizeExceeded => write!(f, "Maximum input size exceeded"),
            Self::MaxStringLengthExceeded => write!(f, "Maximum string length exceeded"),
        }
    }
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(f, "File not found: {}", path),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::ReadError(msg) => write!(f, "Read error: {}", msg),
            Self::WriteError(msg) => write!(f, "Write error: {}", msg),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(Box::as_ref)
    }

    fn description(&self) -> &str {
        match &self.kind {
            ParseErrorKind::Conversion(_) => "conversion error",
            ParseErrorKind::Format(_) => "format error",
            ParseErrorKind::IO(_) => "I/O error",
            ParseErrorKind::Lexical(_) => "lexical error",
            ParseErrorKind::Security(_) => "security error",
            ParseErrorKind::Semantic(_) => "semantic error",
            ParseErrorKind::Syntax(_) => "syntax error",
        }
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn from_lexer(lexer: &Lexer) -> Self {
        let (line, column) = lexer.get_location();
        Self { line, column }
    }

    pub fn create_error(&self, kind: ParseErrorKind, context: &str) -> ParseError {
        let mut error = ParseError::new(kind);
        error = error.with_location(self.line, self.column);

        if !context.is_empty() {
            error = error.with_context(context);
        }

        error
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        let kind = match err.kind() {
            std::io::ErrorKind::NotFound => {
                ParseErrorKind::IO(IOError::FileNotFound(err.to_string()))
            }
            std::io::ErrorKind::PermissionDenied => {
                ParseErrorKind::IO(IOError::PermissionDenied(err.to_string()))
            }
            _ => ParseErrorKind::IO(IOError::ReadError(err.to_string())),
        };

        Self {
            kind,
            location: None,
            source: Some(Box::new(err)), // Store the original error
            context: None,
        }
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self {
            kind: ParseErrorKind::Lexical(LexicalError::InvalidNumber(err.to_string())),
            location: None,
            source: Some(Box::new(err)),
            context: None,
        }
    }
}

impl From<std::num::ParseFloatError> for ParseError {
    fn from(err: std::num::ParseFloatError) -> Self {
        Self {
            kind: ParseErrorKind::Lexical(LexicalError::InvalidNumber(err.to_string())),
            location: None,
            source: Some(Box::new(err)),
            context: None,
        }
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self {
            kind: ParseErrorKind::Lexical(LexicalError::InvalidString(
                "Invalid UTF-8 sequence".to_string(),
            )),
            location: None,
            source: Some(Box::new(err)),
            context: None,
        }
    }
}
