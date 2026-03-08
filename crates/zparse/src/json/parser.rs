//! JSON streaming parser implementation

use crate::error::{Error, ErrorKind, Result};
use crate::json::event::Event;
use crate::lexer::json::JsonLexer;
use crate::lexer::{Token, TokenKind};
use crate::value::{Array, Object, Value};

pub const DEFAULT_MAX_DEPTH: u16 = 128;
pub const DEFAULT_MAX_SIZE: usize = 10 * 1024 * 1024;

/// Configuration for the JSON parser
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Maximum nesting depth (0 means unlimited)
    pub max_depth: u16,
    /// Maximum input size in bytes (0 means unlimited)
    pub max_size: usize,
    /// Allow JavaScript-style comments
    pub allow_comments: bool,
    /// Allow trailing commas in objects and arrays
    pub allow_trailing_commas: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            max_size: DEFAULT_MAX_SIZE,
            allow_comments: false,
            allow_trailing_commas: false,
        }
    }
}

impl Config {
    /// Create a new config with unlimited depth and size
    pub const fn unlimited() -> Self {
        Self {
            max_depth: 0,
            max_size: 0,
            allow_comments: false,
            allow_trailing_commas: false,
        }
    }

    /// Create a new config with specific limits
    pub const fn new(max_depth: u16, max_size: usize) -> Self {
        Self {
            max_depth,
            max_size,
            allow_comments: false,
            allow_trailing_commas: false,
        }
    }

    /// Enable or disable comment support
    pub const fn with_comments(mut self, allow: bool) -> Self {
        self.allow_comments = allow;
        self
    }

    /// Enable or disable trailing comma support
    pub const fn with_trailing_commas(mut self, allow: bool) -> Self {
        self.allow_trailing_commas = allow;
        self
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
    /// Whether we just consumed a comma and expect a key
    expecting_key: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser with default configuration
    pub fn new(input: &'a [u8]) -> Self {
        Self::with_config(input, Config::default())
    }

    /// Create a new parser with custom configuration
    pub fn with_config(input: &'a [u8], config: Config) -> Self {
        Self {
            lexer: JsonLexer::with_options(input, config.allow_comments),
            config,
            depth: 0,
            bytes_parsed: 0,
            context_stack: Vec::new(),
            expecting_colon_after_key: false,
            expecting_value: false,
            is_first_element: true,
            expecting_key: false,
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
                1,
                1,
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
                1,
                1,
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

    /// Returns the parser configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the current parsing depth.
    pub fn depth(&self) -> u16 {
        self.depth
    }

    /// Returns the number of bytes parsed so far.
    pub fn bytes_parsed(&self) -> usize {
        self.bytes_parsed
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
        if self.expecting_key {
            match token.kind {
                TokenKind::RightBrace if self.config.allow_trailing_commas => {
                    self.expecting_key = false;
                    self.pop_context();
                    return Ok(Some(Event::ObjectEnd));
                }
                TokenKind::String(s) => {
                    self.expecting_key = false;
                    self.is_first_element = false;
                    self.expecting_colon_after_key = true;
                    return Ok(Some(Event::Key(s)));
                }
                _ => return Err(self.expected_error("string key", &token)),
            }
        }

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
                self.expecting_key = true;
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
            TokenKind::RightBracket
                if self.expecting_value && self.config.allow_trailing_commas =>
            {
                self.expecting_value = false;
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
                self.expecting_key = false;
                Ok(Some(Event::ObjectStart))
            }
            TokenKind::LeftBracket => {
                self.increment_depth()?;
                self.context_stack.push(ContainerContext::Array);
                self.is_first_element = true;
                self.expecting_colon_after_key = false;
                self.expecting_value = false;
                self.expecting_key = false;
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
                self.bytes_parsed,
                1,
                1,
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
            self.expecting_key = false;
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
