use crate::error::{ParseError, ParseErrorKind, Result};

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
    current_depth: usize,
    current_size: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            max_size: 10 * 1024 * 1024,     // 10MB
            max_string_length: 1024 * 1024, // 1MB
            max_object_entries: 10000,
        }
    }
}

impl ParserConfig {
    pub fn validate_string(&self, s: &str) -> Result<()> {
        if s.len() > self.max_string_length {
            return Err(ParseError::new(ParseErrorKind::MaxStringLengthExceeded));
        }
        Ok(())
    }

    pub fn validate_object_entries(&self, count: usize) -> Result<()> {
        if count > self.max_object_entries {
            return Err(ParseError::new(ParseErrorKind::MaxObjectEntriesExceeded));
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
            return Err(ParseError::new(ParseErrorKind::MaxDepthExceeded));
        }
        Ok(())
    }

    pub fn exit_nested(&mut self) {
        self.current_depth -= 1;
    }

    pub fn add_size(&mut self, size: usize, config: &ParserConfig) -> Result<()> {
        self.current_size += size;
        if self.current_size > config.max_size {
            return Err(ParseError::new(ParseErrorKind::MaxSizeExceeded));
        }
        Ok(())
    }
}
