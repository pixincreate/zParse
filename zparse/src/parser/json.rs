// parser/json.rs
//! JSON parser implementation.
//!
//! This module provides a recursive descent parser for JSON documents that:
//! - Validates JSON syntax
//! - Constructs a type-safe value tree
//! - Provides detailed error messages
//! - Handles nested structures with proper depth checking

use std::collections::HashMap;

use super::{config::ParserConfig, state::ParserState};
use crate::{
    enums::Token,
    error::{
        LexicalError, Location, ParseError, ParseErrorKind, Result, SemanticError, SyntaxError,
    },
    parser::{lexer::Lexer, value::Value},
};

#[derive(Debug)]
pub struct JsonParser {
    /// Lexer that provides tokens
    lexer: Lexer,
    /// Current token being processed
    current_token: Token,
    /// Parser state for tracking context
    state: ParserState,
}

impl JsonParser {
    /// Creates a new JSON parser for the given input
    pub fn new(input: &str) -> Result<Self> {
        if input.is_empty() {
            return Err(ParseError::new(ParseErrorKind::Semantic(
                SemanticError::InvalidFormat,
            )));
        }

        let state = ParserState::new();

        // Check input size first
        state.validate_input_size(input.len())?;

        let mut lexer = Lexer::new_json(input);
        let current_token = lexer.next_token()?;

        // Initialize with default config
        Ok(Self {
            lexer,
            current_token,
            state,
        })
    }

    fn create_error(&self, kind: ParseErrorKind, context: &str) -> ParseError {
        Location::from_lexer(&self.lexer).create_error(kind, context)
    }

    /// Setter method to configure the parser
    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.state = ParserState::with_config(config);
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
            return Err(self.create_error(
                ParseErrorKind::Lexical(LexicalError::UnexpectedToken(format!(
                    "{:?}",
                    self.current_token
                ))),
                "Unexpected trailing content after JSON document",
            ));
        }

        Ok(value)
    }

    /// Parses a JSON value
    fn parse_value(&mut self) -> Result<Value> {
        match &self.current_token {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::String(s) => {
                let value = Value::String(s.clone());
                self.advance()?;
                Ok(value)
            }
            Token::Number(n) => {
                let value = Value::Number(*n);
                self.advance()?;
                Ok(value)
            }
            Token::Boolean(b) => {
                let value = Value::Boolean(*b);
                self.advance()?;
                Ok(value)
            }
            Token::Null => {
                let value = Value::Null;
                self.advance()?;
                Ok(value)
            }
            _ => Err(self.create_error(
                ParseErrorKind::Lexical(LexicalError::UnexpectedToken(format!(
                    "{:?}",
                    self.current_token
                ))),
                "Expected a valid JSON value (object, array, string, number, boolean, or null)",
            )),
        }
    }

    /// Parses a JSON object
    fn parse_object(&mut self) -> Result<Value> {
        // Enter nested context and check depth
        self.state.enter_nested()?;

        let mut map = HashMap::new();
        self.advance()?; // consume '{'

        let mut entry_count = 0;

        // Handle empty object
        if matches!(self.current_token, Token::RightBrace) {
            self.advance()?;
            self.state.exit_nested();
            return Ok(Value::Map(map));
        }

        loop {
            // Validate entry count
            entry_count += 1;
            self.state.validate_object_entries(entry_count)?;

            // Parse key
            let key = match &self.current_token {
                Token::String(s) => {
                    self.state.validate_string(s)?;
                    s.clone()
                }
                _ => {
                    return Err(self.create_error(
                        ParseErrorKind::Lexical(LexicalError::UnexpectedToken(
                            "Expected string key".to_string(),
                        )),
                        "Object keys must be strings",
                    ))
                }
            };
            self.advance()?;

            // Expect colon
            if self.current_token != Token::Colon {
                return Err(self.create_error(
                    ParseErrorKind::Syntax(SyntaxError::MissingColon),
                    "Expected ':' after object key",
                ));
            }
            self.advance()?;

            // Parse value
            let value = self.parse_value()?;

            // Check for duplicate keys
            if map.contains_key(&key) {
                return Err(self.create_error(
                    ParseErrorKind::Syntax(SyntaxError::DuplicateKey(key)),
                    "Duplicate key found in object",
                ));
            }

            map.insert(key, value);

            // Handle comma or end of object
            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                    if matches!(self.current_token, Token::RightBrace) {
                        return Err(self.create_error(
                            ParseErrorKind::Lexical(LexicalError::UnexpectedToken(format!(
                                "Trailing {:?} in object",
                                self.current_token
                            ))),
                            "Trailing comma not allowed in object",
                        ));
                    }
                }
                Token::RightBrace => {
                    self.advance()?;
                    self.state.exit_nested();
                    return Ok(Value::Map(map));
                }
                _ => {
                    return Err(self.create_error(
                        ParseErrorKind::Syntax(SyntaxError::MissingComma),
                        "Expected ',' between object properties",
                    ))
                }
            }
        }
    }

    /// Parses a JSON array
    fn parse_array(&mut self) -> Result<Value> {
        // Enter nested context and check depth
        self.state.enter_nested()?;

        let mut array = Vec::new();
        self.advance()?; // consume '['

        // Handle empty array
        if matches!(self.current_token, Token::RightBracket) {
            self.advance()?;
            self.state.exit_nested();
            return Ok(Value::Array(array));
        }

        loop {
            // Track array size
            self.state.add_size(1)?;

            // Parse array element
            let value = self.parse_value()?;
            array.push(value);

            // Handle comma or end of array
            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                    if matches!(self.current_token, Token::RightBracket) {
                        return Err(self.create_error(
                            ParseErrorKind::Lexical(LexicalError::UnexpectedToken(format!(
                                "Trailing {:?} in array",
                                self.current_token
                            ))),
                            "Trailing comma not allowed in array",
                        ));
                    }
                }
                Token::RightBracket => {
                    self.advance()?;
                    self.state.exit_nested();
                    return Ok(Value::Array(array));
                }
                _ => {
                    return Err(self.create_error(
                        ParseErrorKind::Lexical(LexicalError::UnexpectedToken(format!(
                            "{:?}. Expected , or ]",
                            self.current_token
                        ))),
                        "Expected ',' between array elements or ']' to close array",
                    ))
                }
            }
        }
    }
}
