//! JSON streaming parser implementation

use crate::error::{Error, ErrorKind, Result};
use crate::json::event::Event;
use crate::lexer::json::JsonLexer;
use crate::lexer::{Token, TokenKind};
use crate::value::{Array, Object, Value};

/// Configuration for the JSON parser
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Maximum nesting depth (0 means unlimited)
    pub max_depth: u16,
    /// Maximum input size in bytes (0 means unlimited)
    pub max_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_depth: 128,
            max_size: 10 * 1024 * 1024, // 10 MB default
        }
    }
}

impl Config {
    /// Create a new config with unlimited depth and size
    pub const fn unlimited() -> Self {
        Self {
            max_depth: 0,
            max_size: 0,
        }
    }

    /// Create a new config with specific limits
    pub const fn new(max_depth: u16, max_size: usize) -> Self {
        Self {
            max_depth,
            max_size,
        }
    }
}

/// Context for tracking position within containers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerContext {
    /// Inside an object, expecting a key or end
    Object,
    /// Inside an array, expecting a value or end
    Array,
}

/// Streaming JSON parser with depth and size limits
#[derive(Debug)]
pub struct Parser<'a> {
    lexer: JsonLexer<'a>,
    config: Config,
    depth: u16,
    bytes_parsed: usize,
    /// Stack of container contexts to track where we are
    context_stack: Vec<ContainerContext>,
    /// Whether we just emitted a key and are expecting a colon
    expecting_colon_after_key: bool,
    /// Whether we're expecting a value (after colon in object, or in array)
    expecting_value: bool,
    /// Whether we're in the first element of a container
    is_first_element: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser with default configuration
    pub fn new(input: &'a [u8]) -> Self {
        Self::with_config(input, Config::default())
    }

    /// Create a new parser with custom configuration
    pub fn with_config(input: &'a [u8], config: Config) -> Self {
        Self {
            lexer: JsonLexer::new(input),
            config,
            depth: 0,
            bytes_parsed: 0,
            context_stack: Vec::new(),
            expecting_colon_after_key: false,
            expecting_value: false,
            is_first_element: true,
        }
    }

    /// Get the next event from the parser
    pub fn next_event(&mut self) -> Result<Option<Event>> {
        // Check size limit before processing
        if self.config.max_size > 0 && self.bytes_parsed >= self.config.max_size {
            return Err(Error::at(
                ErrorKind::MaxSizeExceeded {
                    max: self.config.max_size,
                },
                self.bytes_parsed,
                0,
                0,
            ));
        }

        // Skip whitespace and get next token
        let token = self.lexer.next_token()?;

        // Track bytes parsed based on token span
        let span = token.span;
        let token_len = span.end.offset.saturating_sub(span.start.offset);
        self.bytes_parsed = self.bytes_parsed.saturating_add(token_len);

        // Check size limit after updating
        if self.config.max_size > 0 && self.bytes_parsed > self.config.max_size {
            return Err(Error::at(
                ErrorKind::MaxSizeExceeded {
                    max: self.config.max_size,
                },
                self.bytes_parsed,
                0,
                0,
            ));
        }

        // Handle EOF at root level
        if token.kind == TokenKind::Eof && self.context_stack.is_empty() {
            return Ok(None);
        }

        // Determine current context
        let current_context = self.context_stack.last().copied();

        match current_context {
            None => self.handle_root(token),
            Some(ContainerContext::Object) => self.handle_in_object(token),
            Some(ContainerContext::Array) => self.handle_in_array(token),
        }
    }

    /// Parse the complete input into a Value
    pub fn parse_value(&mut self) -> Result<Value> {
        let mut object_stack: Vec<Object> = Vec::new();
        let mut array_stack: Vec<Array> = Vec::new();
        // Stack to track keys for nested objects
        let mut key_stack: Vec<Option<String>> = Vec::new();
        let mut current_key: Option<String> = None;

        while let Some(event) = self.next_event()? {
            match event {
                Event::ObjectStart => {
                    // Save current key context before entering new object
                    key_stack.push(current_key.take());
                    object_stack.push(Object::new());
                }
                Event::ObjectEnd => {
                    let obj = object_stack
                        .pop()
                        .ok_or_else(|| self.error(ErrorKind::InvalidToken))?;
                    // Restore the key context for this object
                    let obj_key = key_stack.pop().flatten();

                    if let Some(key) = obj_key {
                        if let Some(parent_obj) = object_stack.last_mut() {
                            parent_obj.insert(key, Value::Object(obj));
                        } else if let Some(parent_arr) = array_stack.last_mut() {
                            parent_arr.push(Value::Object(obj));
                        } else {
                            return Ok(Value::Object(obj));
                        }
                    } else if let Some(parent_arr) = array_stack.last_mut() {
                        parent_arr.push(Value::Object(obj));
                    } else {
                        return Ok(Value::Object(obj));
                    }
                }
                Event::ArrayStart => {
                    // Save current key context before entering new array
                    key_stack.push(current_key.take());
                    array_stack.push(Array::new());
                }
                Event::ArrayEnd => {
                    let arr = array_stack
                        .pop()
                        .ok_or_else(|| self.error(ErrorKind::InvalidToken))?;
                    // Restore the key context for this array
                    let arr_key = key_stack.pop().flatten();

                    if let Some(key) = arr_key {
                        if let Some(parent_obj) = object_stack.last_mut() {
                            parent_obj.insert(key, Value::Array(arr));
                        } else if let Some(parent_arr) = array_stack.last_mut() {
                            parent_arr.push(Value::Array(arr));
                        } else {
                            return Ok(Value::Array(arr));
                        }
                    } else if let Some(parent_arr) = array_stack.last_mut() {
                        parent_arr.push(Value::Array(arr));
                    } else {
                        return Ok(Value::Array(arr));
                    }
                }
                Event::Key(key) => {
                    current_key = Some(key);
                }
                Event::Value(value) => {
                    if let Some(key) = current_key.take() {
                        if let Some(parent_obj) = object_stack.last_mut() {
                            parent_obj.insert(key, value);
                        } else {
                            return Ok(value);
                        }
                    } else if let Some(parent_arr) = array_stack.last_mut() {
                        parent_arr.push(value);
                    } else {
                        return Ok(value);
                    }
                }
            }
        }

        // If we get here without returning, check if there's anything left on stacks
        if let Some(obj) = object_stack.pop() {
            if object_stack.is_empty() && array_stack.is_empty() {
                return Ok(Value::Object(obj));
            }
        }
        if let Some(arr) = array_stack.pop() {
            if object_stack.is_empty() && array_stack.is_empty() {
                return Ok(Value::Array(arr));
            }
        }

        Err(self.error(ErrorKind::InvalidToken))
    }

    // Helper methods

    fn handle_root(&mut self, token: Token) -> Result<Option<Event>> {
        match token.kind {
            TokenKind::LeftBrace => {
                self.increment_depth()?;
                self.context_stack.push(ContainerContext::Object);
                self.is_first_element = true;
                Ok(Some(Event::ObjectStart))
            }
            TokenKind::LeftBracket => {
                self.increment_depth()?;
                self.context_stack.push(ContainerContext::Array);
                self.is_first_element = true;
                Ok(Some(Event::ArrayStart))
            }
            TokenKind::Null => Ok(Some(Event::Value(Value::Null))),
            TokenKind::True => Ok(Some(Event::Value(Value::Bool(true)))),
            TokenKind::False => Ok(Some(Event::Value(Value::Bool(false)))),
            TokenKind::String(s) => Ok(Some(Event::Value(Value::String(s)))),
            TokenKind::Number(n) => Ok(Some(Event::Value(Value::Number(n)))),
            TokenKind::Eof => Ok(None),
            _ => Err(self.expected_error("value", &token)),
        }
    }

    fn handle_in_object(&mut self, token: Token) -> Result<Option<Event>> {
        // Handle colon after key
        if self.expecting_colon_after_key {
            match token.kind {
                TokenKind::Colon => {
                    // Consume colon and get the value in next call
                    self.expecting_colon_after_key = false;
                    self.expecting_value = true;
                    return self.next_event();
                }
                _ => {
                    return Err(self.expected_error("':'", &token));
                }
            }
        }

        // If we're expecting a value, parse it
        if self.expecting_value {
            self.expecting_value = false;
            self.is_first_element = false;
            return self.parse_value_token(token);
        }

        match token.kind {
            TokenKind::RightBrace => {
                self.pop_context();
                Ok(Some(Event::ObjectEnd))
            }
            TokenKind::String(s) if self.is_first_element || self.expect_comma() => {
                // This is a key
                self.is_first_element = false;
                self.expecting_colon_after_key = true;
                Ok(Some(Event::Key(s)))
            }
            TokenKind::Comma if !self.is_first_element && !self.expecting_colon_after_key => {
                // Comma is valid here, continue to next token
                self.next_event()
            }
            _ => {
                if self.is_first_element {
                    Err(self.expected_error("string key or '}'", &token))
                } else {
                    Err(self.expected_error("',' or '}'", &token))
                }
            }
        }
    }

    fn handle_in_array(&mut self, token: Token) -> Result<Option<Event>> {
        match token.kind {
            TokenKind::RightBracket if !self.expecting_value => {
                self.pop_context();
                Ok(Some(Event::ArrayEnd))
            }
            TokenKind::Comma if !self.is_first_element && !self.expecting_value => {
                // Comma is valid, now we expect a value
                self.expecting_value = true;
                self.next_event()
            }
            _ if self.is_first_element || self.expecting_value || self.expect_comma() => {
                self.is_first_element = false;
                self.expecting_value = false;
                self.parse_value_token(token)
            }
            _ => Err(self.expected_error("value or ']'", &token)),
        }
    }

    fn parse_value_token(&mut self, token: Token) -> Result<Option<Event>> {
        match token.kind {
            TokenKind::LeftBrace => {
                self.increment_depth()?;
                self.context_stack.push(ContainerContext::Object);
                self.is_first_element = true;
                self.expecting_colon_after_key = false;
                self.expecting_value = false;
                Ok(Some(Event::ObjectStart))
            }
            TokenKind::LeftBracket => {
                self.increment_depth()?;
                self.context_stack.push(ContainerContext::Array);
                self.is_first_element = true;
                self.expecting_colon_after_key = false;
                self.expecting_value = false;
                Ok(Some(Event::ArrayStart))
            }
            TokenKind::Null => {
                self.expecting_value = false;
                Ok(Some(Event::Value(Value::Null)))
            }
            TokenKind::True => {
                self.expecting_value = false;
                Ok(Some(Event::Value(Value::Bool(true))))
            }
            TokenKind::False => {
                self.expecting_value = false;
                Ok(Some(Event::Value(Value::Bool(false))))
            }
            TokenKind::String(s) => {
                self.expecting_value = false;
                Ok(Some(Event::Value(Value::String(s))))
            }
            TokenKind::Number(n) => {
                self.expecting_value = false;
                Ok(Some(Event::Value(Value::Number(n))))
            }
            _ => Err(self.expected_error("value", &token)),
        }
    }

    fn expect_comma(&self) -> bool {
        // We expect a comma if we're not at the first element and not expecting a colon or value
        !self.is_first_element && !self.expecting_colon_after_key && !self.expecting_value
    }

    fn increment_depth(&mut self) -> Result<()> {
        if self.config.max_depth > 0 && self.depth >= self.config.max_depth {
            return Err(Error::at(
                ErrorKind::MaxDepthExceeded {
                    max: self.config.max_depth,
                },
                0,
                0,
                0,
            ));
        }
        self.depth = self.depth.saturating_add(1);
        Ok(())
    }

    fn pop_context(&mut self) {
        self.context_stack.pop();
        self.depth = self.depth.saturating_sub(1);
        // Reset state for next element in parent container
        if !self.context_stack.is_empty() {
            self.is_first_element = false;
            self.expecting_colon_after_key = false;
            self.expecting_value = false;
        }
    }

    fn error(&self, kind: ErrorKind) -> Error {
        Error::at(kind, self.bytes_parsed, 0, 0)
    }

    fn expected_error(&self, expected: &str, token: &Token) -> Error {
        let found = token.kind.name();
        Error::at(
            ErrorKind::Expected {
                expected: expected.to_string(),
                found: found.to_string(),
            },
            token.span.start.offset,
            token.span.start.line,
            token.span.start.col,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Error, ErrorKind, Result, Span};
    use std::fmt::Debug;

    fn fail<T>(message: String) -> Result<T> {
        Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            message,
        ))
    }

    fn ensure_eq<T: PartialEq + Debug>(left: T, right: T) -> Result<()> {
        if left == right {
            Ok(())
        } else {
            fail(format!("assertion failed: left={left:?} right={right:?}"))
        }
    }

    fn next_event_or_fail(parser: &mut Parser<'_>) -> Result<Option<Event>> {
        parser.next_event()
    }

    fn parse_value_or_fail(parser: &mut Parser<'_>) -> Result<Value> {
        parser.parse_value()
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.max_depth, 128);
        assert_eq!(config.max_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_config_unlimited() {
        let config = Config::unlimited();
        assert_eq!(config.max_depth, 0);
        assert_eq!(config.max_size, 0);
    }

    #[test]
    fn test_config_new() {
        let config = Config::new(64, 1024);
        assert_eq!(config.max_depth, 64);
        assert_eq!(config.max_size, 1024);
    }

    #[test]
    fn test_parser_new() {
        let input = b"null";
        let parser = Parser::new(input);
        assert_eq!(parser.config.max_depth, 128);
        assert_eq!(parser.depth, 0);
        assert_eq!(parser.bytes_parsed, 0);
    }

    #[test]
    fn test_parser_with_config() {
        let input = b"null";
        let config = Config::new(32, 512);
        let parser = Parser::with_config(input, config);
        assert_eq!(parser.config.max_depth, 32);
        assert_eq!(parser.config.max_size, 512);
    }

    #[test]
    fn test_parse_null() -> Result<()> {
        let input = b"null";
        let mut parser = Parser::new(input);

        let event = next_event_or_fail(&mut parser);
        let event = event?;
        ensure_eq(event, Some(Event::Value(Value::Null)))?;

        let event = next_event_or_fail(&mut parser);
        let event = event?;
        ensure_eq(event, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_bool() -> Result<()> {
        let input = b"true";
        let mut parser = Parser::new(input);

        let event = next_event_or_fail(&mut parser);
        let event = event?;
        ensure_eq(event, Some(Event::Value(Value::Bool(true))))?;
        Ok(())
    }

    #[test]
    fn test_parse_number() -> Result<()> {
        let input = b"42.5";
        let mut parser = Parser::new(input);

        let event = next_event_or_fail(&mut parser);
        let event = event?;
        ensure_eq(event, Some(Event::Value(Value::Number(42.5))))?;
        Ok(())
    }

    #[test]
    fn test_parse_string() -> Result<()> {
        let input = br#""hello world""#;
        let mut parser = Parser::new(input);

        let event = next_event_or_fail(&mut parser)?;
        ensure_eq(
            event,
            Some(Event::Value(Value::String("hello world".to_string()))),
        )?;
        Ok(())
    }

    #[test]
    fn test_parse_empty_object() -> Result<()> {
        let input = b"{}";
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_empty_array() -> Result<()> {
        let input = b"[]";
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_simple_object() -> Result<()> {
        let input = br#"{"key": "value"}"#;
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("key".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::String("value".to_string()))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_simple_array() -> Result<()> {
        let input = b"[1, 2, 3]";
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(1.0))),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(2.0))),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(3.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_nested_object() -> Result<()> {
        let input = br#"{"outer": {"inner": 42}}"#;
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("outer".to_string())),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("inner".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(42.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_mixed() -> Result<()> {
        let input = br#"{"name": "test", "values": [1, 2], "flag": true}"#;
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("name".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::String("test".to_string()))),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("values".to_string())),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(1.0))),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(2.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("flag".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Bool(true))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_value_null() -> Result<()> {
        let input = b"null";
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;
        ensure_eq(value, Value::Null)?;
        Ok(())
    }

    #[test]
    fn test_parse_value_bool() -> Result<()> {
        let input = b"true";
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;
        ensure_eq(value, Value::Bool(true))?;
        Ok(())
    }

    #[test]
    fn test_parse_value_number() -> Result<()> {
        let input = b"123.456";
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;
        ensure_eq(value, Value::Number(123.456))?;
        Ok(())
    }

    #[test]
    fn test_parse_value_string() -> Result<()> {
        let input = br#""test string""#;
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;
        ensure_eq(value, Value::String("test string".to_string()))?;
        Ok(())
    }

    #[test]
    fn test_parse_value_array() -> Result<()> {
        let input = b"[1, 2, 3]";
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;
        let expected = Value::Array(vec![1.0.into(), 2.0.into(), 3.0.into()].into());
        ensure_eq(value, expected)?;
        Ok(())
    }

    #[test]
    fn test_parse_value_object() -> Result<()> {
        let input = br#"{"a": 1, "b": 2}"#;
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;

        let mut expected = Object::new();
        expected.insert("a", 1i32);
        expected.insert("b", 2i32);
        ensure_eq(value, Value::Object(expected))?;
        Ok(())
    }

    #[test]
    fn test_parse_value_nested() -> Result<()> {
        let input = br#"{"arr": [1, {"nested": "value"}]}"#;
        let mut parser = Parser::new(input);
        let value = parse_value_or_fail(&mut parser)?;

        if let Value::Object(obj) = value {
            if !obj.contains_key("arr") {
                return fail("Expected key 'arr'".to_string());
            }
            if let Some(Value::Array(arr)) = obj.get("arr") {
                ensure_eq(arr.len(), 2)?;
                ensure_eq(arr.get(0), Some(&Value::Number(1.0)))?;
            } else {
                return fail("Expected array".to_string());
            }
        } else {
            return fail("Expected object".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_depth_limit() -> Result<()> {
        let input = br#"{"a": {"b": {"c": 1}}}"#;
        let config = Config::new(2, 0); // max depth of 2
        let mut parser = Parser::with_config(input, config);

        // Should fail when trying to enter third level
        let result = parser.parse_value();
        if !matches!(
            result,
            Err(err) if matches!(err.kind(), ErrorKind::MaxDepthExceeded { max: 2 })
        ) {
            return fail("Expected max depth error".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_size_limit() -> Result<()> {
        let input = b"1234567890";
        let config = Config::new(0, 5); // max size of 5 bytes
        let mut parser = Parser::with_config(input, config);

        let result = parser.parse_value();
        if !matches!(
            result,
            Err(err) if matches!(err.kind(), ErrorKind::MaxSizeExceeded { max: 5 })
        ) {
            return fail("Expected max size error".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_object_with_multiple_keys() -> Result<()> {
        let input = br#"{"a": 1, "b": 2, "c": 3}"#;
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("a".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(1.0))),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("b".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(2.0))),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("c".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(3.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_array_with_nested_objects() -> Result<()> {
        let input = br#"[{"x": 1}, {"y": 2}]"#;
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("x".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(1.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Key("y".to_string())),
        )?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(2.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }

    #[test]
    fn test_parse_deeply_nested() -> Result<()> {
        let input = br#"[[[[1]]]]"#;
        let mut parser = Parser::new(input);

        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
        ensure_eq(
            next_event_or_fail(&mut parser)?,
            Some(Event::Value(Value::Number(1.0))),
        )?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
        ensure_eq(next_event_or_fail(&mut parser)?, None)?;
        Ok(())
    }
}
