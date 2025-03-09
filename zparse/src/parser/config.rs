use std::fmt;

use crate::error::{ParseError, ParseErrorKind, Result, SecurityError};

/// Maximum nesting depth (32) based on common JSON/TOML usage patterns
pub const DEFAULT_MAX_DEPTH: usize = 32;
/// Maximum input size (1MB) to prevent memory exhaustion attacks
pub const DEFAULT_MAX_SIZE: usize = 1_048_576; // 1MB
/// Maximum string length (100KB) to prevent buffer overflows
pub const DEFAULT_MAX_STRING_LENGTH: usize = 102_400; // 100KB
/// Maximum number of object entries (1K) to prevent abuse
pub const DEFAULT_MAX_OBJECT_ENTRIES: usize = 1_000;

/// Configuration for parser limits and validation
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum nesting depth for objects/arrays
    pub max_depth: usize,
    /// Maximum input size in bytes
    pub max_size: usize,
    /// Maximum string length
    pub max_string_length: usize,
    /// Maximum number of object entries
    pub max_object_entries: usize,
}

/// Tracks memory usage and nesting depth during parsing
#[derive(Debug)]
pub struct ParsingContext {
    pub current_depth: usize,
    current_size: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            max_size: DEFAULT_MAX_SIZE,
            max_string_length: DEFAULT_MAX_STRING_LENGTH,
            max_object_entries: DEFAULT_MAX_OBJECT_ENTRIES,
        }
    }
}

impl fmt::Display for ParserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParserConfig {{ max_depth: {}, max_size: {}, max_string_length: {}, max_object_entries: {} }}",
            self.max_depth, self.max_size, self.max_string_length, self.max_object_entries
        )
    }
}

impl ParserConfig {
    pub fn validate_string(&self, s: &str) -> Result<()> {
        if s.len() > self.max_string_length {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxStringLengthExceeded,
            )));
        }
        Ok(())
    }

    pub fn validate_object_entries(&self, count: usize) -> Result<()> {
        if count > self.max_object_entries {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxObjectEntriesExceeded,
            )));
        }
        Ok(())
    }
}

impl Default for ParsingContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsingContext {
    pub fn new() -> Self {
        Self {
            current_depth: 0,
            current_size: 0,
        }
    }

    pub fn enter_nested(&mut self, config: &ParserConfig) -> Result<()> {
        self.current_depth += 1;
        if self.current_depth > config.max_depth {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxDepthExceeded,
            )));
        }
        Ok(())
    }

    pub fn exit_nested(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    pub fn add_size(&mut self, size: usize, config: &ParserConfig) -> Result<()> {
        self.current_size += size;
        if self.current_size > config.max_size {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxSizeExceeded,
            )));
        }
        Ok(())
    }
}
