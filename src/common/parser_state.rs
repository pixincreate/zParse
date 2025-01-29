use crate::error::Result;
use crate::error::{ParseError, ParseErrorKind, SecurityError};
use crate::parser::config::{ParserConfig, ParsingContext};

#[derive(Debug)]
pub struct ParserState {
    pub config: ParserConfig,
    pub context: ParsingContext,
}

impl Default for ParserState {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserState {
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
            context: ParsingContext::new(),
        }
    }

    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            config,
            context: ParsingContext::new(),
        }
    }

    pub fn enter_nested(&mut self) -> Result<()> {
        self.context.enter_nested(&self.config)
    }

    pub fn exit_nested(&mut self) {
        self.context.exit_nested()
    }

    pub fn validate_string(&self, s: &str) -> Result<()> {
        self.config.validate_string(s)
    }

    pub fn validate_object_entries(&self, count: usize) -> Result<()> {
        if count > self.config.max_object_entries {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxObjectEntriesExceeded,
            )));
        }
        Ok(())
    }

    pub fn add_size(&mut self, size: usize) -> Result<()> {
        self.context.add_size(size, &self.config)
    }

    pub fn validate_input_size(&self, size: usize) -> Result<()> {
        if size > self.config.max_size {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxSizeExceeded,
            )));
        }
        Ok(())
    }
}
