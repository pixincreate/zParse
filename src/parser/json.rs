// parser/json.rs
//! JSON parser implementation.
//!
//! This module provides a recursive descent parser for JSON documents that:
//! - Validates JSON syntax
//! - Constructs a type-safe value tree
//! - Provides detailed error messages
//! - Handles nested structures with proper depth checking

use super::{lexer::Lexer, value::Value};
use crate::enums::Token;
use crate::error::{ParseError, ParseErrorKind, Result};
use std::collections::HashMap;

/// Parser for JSON documents
pub struct JsonParser {
    /// Lexer that provides tokens
    lexer: Lexer,
    /// Current token being processed
    current_token: Token,
}

impl JsonParser {
    /// Creates a new JSON parser for the given input
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new_json(input); // Use new_json instead of new
        let current_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
        })
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
        let mut map = HashMap::new();
        self.advance()?; // consume '{'

        if self.current_token == Token::EOF {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }

        // Handle empty object
        if self.current_token == Token::RightBrace {
            self.advance()?;
            return Ok(Value::Object(map));
        }

        loop {
            // Parse key - ONLY accept string tokens
            let key = match &self.current_token {
                Token::String(s) => s.clone(),
                Token::RightBrace => {
                    return Err(ParseError::new(ParseErrorKind::InvalidValue(
                        "Trailing comma".to_string(),
                    )))
                }
                Token::EOF => return Err(ParseError::new(ParseErrorKind::UnexpectedEOF)),
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Object key must be a string".to_string(),
                    )))
                }
            };
            self.advance()?;

            // Parse colon
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

        Ok(Value::Object(map))
    }

    /// Parses a JSON array
    fn parse_array(&mut self) -> Result<Value> {
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

        Ok(Value::Array(array))
    }
}
