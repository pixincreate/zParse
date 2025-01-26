use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    location: Option<Location>,
    source: Option<Box<dyn Error>>,
}

#[derive(Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    InvalidToken(String),
    UnexpectedToken(String),
    UnexpectedEOF,
    InvalidNumber(String),
    InvalidString(String),
    InvalidBoolean,
    InvalidDateTime,
    InvalidEscape(char),
    InvalidUnicode,
    InvalidValue(String),
    InvalidKey(String),
    NestedTableError,
    IoError(String),
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
