//! Error types for zparse

use std::fmt;
use thiserror::Error;

/// Position in source code
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pos {
    pub offset: usize,
    pub line: u32,
    pub col: u32,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.offset, self.line, self.col)
    }
}

impl Pos {
    pub const fn new(offset: usize, line: u32, col: u32) -> Self {
        Self { offset, line, col }
    }
}

/// Span representing a range in source code
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub const fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    pub const fn empty() -> Self {
        Self {
            start: Pos::new(0, 0, 0),
            end: Pos::new(0, 0, 0),
        }
    }
}

/// Error kind for detailed categorization
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidEscapeSequence,
    InvalidUnicodeEscape,
    UnterminatedString,
    InvalidNumber,
    InvalidToken,
    Expected { expected: String, found: String },
    TrailingComma,
    MissingComma,
    DuplicateKey { key: String },
    MaxDepthExceeded { max: u16 },
    MaxSizeExceeded { max: usize },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEscapeSequence => write!(f, "invalid escape sequence"),
            Self::InvalidUnicodeEscape => write!(f, "invalid unicode escape"),
            Self::UnterminatedString => write!(f, "unterminated string"),
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidToken => write!(f, "invalid token"),
            Self::Expected { expected, found } => {
                write!(f, "expected {expected}, found {found}")
            }
            Self::TrailingComma => write!(f, "trailing comma"),
            Self::MissingComma => write!(f, "missing comma"),
            Self::DuplicateKey { key } => write!(f, "duplicate key: {key}"),
            Self::MaxDepthExceeded { max } => {
                write!(f, "max depth exceeded: {max}")
            }
            Self::MaxSizeExceeded { max } => write!(f, "max size exceeded: {max}"),
        }
    }
}

/// Main error type for zparse
#[derive(Error, Clone, Debug, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    span: Span,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        let message = kind.to_string();
        Self {
            kind,
            span,
            message,
        }
    }

    pub fn with_message(kind: ErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Create error at specific position
    pub fn at(kind: ErrorKind, offset: usize, line: u32, col: u32) -> Self {
        let pos = Pos::new(offset, line, col);
        Self::new(kind, Span::new(pos, pos))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error at {}: {}", self.span.start, self.message)
    }
}

/// Result type alias for zparse
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_display() {
        let pos = Pos::new(42, 10, 5);
        assert_eq!(pos.to_string(), "42:10:5");
    }

    #[test]
    fn test_error_creation() {
        let err = Error::at(ErrorKind::InvalidToken, 0, 1, 1);
        assert_eq!(err.kind(), &ErrorKind::InvalidToken);
    }

    #[test]
    fn test_error_display() {
        let err = Error::at(ErrorKind::InvalidEscapeSequence, 10, 2, 5);
        let display = err.to_string();
        assert!(display.contains("error at"));
        assert!(display.contains("invalid escape sequence"));
    }
}
