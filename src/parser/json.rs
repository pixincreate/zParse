// parser/json.rs
//! JSON parser implementation.
//!
//! This module provides a recursive descent parser for JSON documents that:
//! - Validates JSON syntax
//! - Constructs a type-safe value tree
//! - Provides detailed error messages
//! - Handles nested structures with proper depth checking

use super::{
    config::{ParserConfig, ParsingContext},
    lexer::Lexer,
    value::Value,
};
use crate::enums::Token;
use crate::error::{ParseError, ParseErrorKind, Result};
use std::collections::HashMap;

/// Parser for JSON documents
pub struct JsonParser {
    /// Lexer that provides tokens
    lexer: Lexer,
    /// Current token being processed
    current_token: Token,
    /// Parser configuration
    config: ParserConfig,
    /// Parsing context for tracking depth and size
    context: ParsingContext,
}

impl JsonParser {
    /// Creates a new JSON parser for the given input
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new_json(input);
        let current_token = lexer.next_token()?;

        // Initialize with default config
        Ok(Self {
            lexer,
            current_token,
            config: ParserConfig::default(),
            context: ParsingContext::new(),
        })
    }

    /// Setter method to configure the parser
    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.config = config;
        self
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    /// Parses a complete JSON document
    /// # Returns
    /// - Ok(Value) containing the parsed document structure
    /// - Err if the input is not valid JSON
    pub fn parse(&mut self) -> Result<Value> {
        let value = self.parse_value()?;

        // Check for trailing content
        if self.current_token != Token::EOF {
            return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                "Unexpected trailing content".to_string(),
            )));
        }

        Ok(value)
    }

    /// Parses a JSON value
    fn parse_value(&mut self) -> Result<Value> {
        match self.current_token {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::String(ref s) => {
                let value = Value::String(s.clone());
                self.advance()?;
                Ok(value)
            }
            Token::Number(n) => {
                let value = Value::Number(n);
                self.advance()?;
                Ok(value)
            }
            Token::Boolean(b) => {
                let value = Value::Boolean(b);
                self.advance()?;
                Ok(value)
            }
            Token::Null => {
                let value = Value::Null;
                self.advance()?;
                Ok(value)
            }
            _ => Err(ParseError::new(ParseErrorKind::UnexpectedToken(format!(
                "{:?}",
                self.current_token
            )))),
        }
    }

    /// Parses a JSON object
    fn parse_object(&mut self) -> Result<Value> {
        // Track nested depth
        self.context.enter_nested(&self.config)?;

        let mut map = HashMap::new();
        self.advance()?; // consume '{'

        if self.current_token == Token::EOF {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }

        // Handle empty object
        if self.current_token == Token::RightBrace {
            self.advance()?;
            self.context.exit_nested();
            return Ok(Value::Object(map));
        }

        // Track number of entries
        let mut entry_count = 0;

        loop {
            // Validate entry count
            entry_count += 1;
            self.config.validate_object_entries(entry_count)?;

            // Parse key and value...
            let key = self.parse_string()?;
            // Validate string length
            self.config.validate_string(&key)?;

            // Expect colon
            if self.current_token != Token::Colon {
                return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                    "Expected colon".to_string(),
                )));
            }
            self.advance()?;

            if self.current_token == Token::EOF {
                return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
            }

            // Parse value
            let value = self.parse_value()?;
            map.insert(key, value);

            // Handle comma or end of object
            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                    if self.current_token == Token::RightBrace {
                        return Err(ParseError::new(ParseErrorKind::InvalidValue(
                            "Trailing comma".to_string(),
                        )));
                    }
                }
                Token::RightBrace => {
                    self.advance()?;
                    break;
                }
                Token::EOF => return Err(ParseError::new(ParseErrorKind::UnexpectedEOF)),
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Expected comma or }".to_string(),
                    )))
                }
            }
        }

        self.context.exit_nested();
        Ok(Value::Object(map))
    }

    /// Parses a JSON array
    fn parse_array(&mut self) -> Result<Value> {
        self.context.enter_nested(&self.config)?;
        let mut array = Vec::new();
        self.advance()?; // consume '['

        // Handle empty array
        if self.current_token == Token::RightBracket {
            self.advance()?;
            return Ok(Value::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                    if self.current_token == Token::RightBracket {
                        return Err(ParseError::new(ParseErrorKind::InvalidValue(
                            "Trailing comma".to_string(),
                        )));
                    }
                }
                Token::RightBracket => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Expected comma or ]".to_string(),
                    )))
                }
            }
        }

        self.context.exit_nested();
        Ok(Value::Array(array))
    }

    /// Parses a JSON string
    fn parse_string(&mut self) -> Result<String> {
        match self.current_token {
            Token::String(ref s) => {
                // Validate string length
                self.config.validate_string(s)?;

                // Track memory usage
                self.context.add_size(s.len(), &self.config)?;

                let value = s.clone();
                self.advance()?;
                Ok(value)
            }
            _ => Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                "Expected string".to_string(),
            ))),
        }
    }
}
