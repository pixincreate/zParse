//! YAML streaming parser implementation

use std::collections::VecDeque;

use crate::error::{Error, ErrorKind, Result, Span};
use crate::lexer::yaml::{YamlLexer, YamlToken, YamlTokenKind};
use crate::value::{Array, Object, Value};
use crate::yaml::event::Event;

/// Configuration for YAML parser
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Maximum nesting depth (0 means unlimited)
    pub max_depth: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_depth: 128 }
    }
}

impl Config {
    pub const fn new(max_depth: u16) -> Self {
        Self { max_depth }
    }
}

/// YAML parser
#[derive(Debug)]
pub struct Parser<'a> {
    lexer: YamlLexer<'a>,
    buffered: Option<YamlToken>,
    config: Config,
    depth: u16,
    events: VecDeque<Event>,
    parsed: Option<Value>,
}

impl<'a> Parser<'a> {
    /// Create a new parser with default config
    pub fn new(input: &'a [u8]) -> Self {
        Self::with_config(input, Config::default())
    }

    /// Create a new parser with custom config
    pub fn with_config(input: &'a [u8], config: Config) -> Self {
        Self {
            lexer: YamlLexer::new(input),
            buffered: None,
            config,
            depth: 0,
            events: VecDeque::new(),
            parsed: None,
        }
    }

    /// Parse entire document
    pub fn parse(&mut self) -> Result<Value> {
        let token = self.peek_non_newline()?;
        if token.kind == YamlTokenKind::Eof {
            self.parsed = Some(Value::Null);
            return Ok(Value::Null);
        }

        let value = self.parse_block()?;
        self.parsed = Some(value.clone());
        Ok(value)
    }

    /// Get next event
    pub fn next_event(&mut self) -> Result<Option<Event>> {
        if let Some(event) = self.events.pop_front() {
            return Ok(Some(event));
        }

        if self.parsed.is_none() {
            let value = self.parse()?;
            emit_events(&value, &mut self.events);
        }

        Ok(self.events.pop_front())
    }

    fn next_token(&mut self) -> Result<YamlToken> {
        let token = match self.buffered.take() {
            Some(token) => token,
            None => self.lexer.next_token()?,
        };
        Ok(token)
    }

    fn peek_token(&mut self) -> Result<YamlToken> {
        if self.buffered.is_none() {
            self.buffered = Some(self.lexer.next_token()?);
        }
        self.buffered.clone().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "missing buffered token".to_string(),
            )
        })
    }

    fn next_non_newline(&mut self) -> Result<YamlToken> {
        loop {
            let token = self.next_token()?;
            match token.kind {
                YamlTokenKind::Newline => continue,
                _ => return Ok(token),
            }
        }
    }

    fn peek_non_newline(&mut self) -> Result<YamlToken> {
        let token = self.next_non_newline()?;
        self.buffered = Some(token.clone());
        Ok(token)
    }

    fn parse_block(&mut self) -> Result<Value> {
        let token = self.peek_non_newline()?;
        match token.kind {
            YamlTokenKind::Dash => self.parse_sequence(),
            YamlTokenKind::Scalar(_) => self.parse_mapping_or_scalar(),
            YamlTokenKind::LeftBracket => self.parse_flow_sequence(),
            YamlTokenKind::LeftBrace => self.parse_flow_mapping(),
            _ => Err(Error::with_message(
                ErrorKind::InvalidToken,
                token.span,
                "expected mapping or sequence".to_string(),
            )),
        }
    }

    fn parse_sequence(&mut self) -> Result<Value> {
        self.bump_depth()?;
        let mut items = Vec::new();

        loop {
            let token = self.next_non_newline()?;
            match token.kind {
                YamlTokenKind::Dash => {
                    let value = self.parse_sequence_item()?;
                    items.push(value);
                }
                YamlTokenKind::Dedent => {
                    self.buffered = Some(token);
                    break;
                }
                YamlTokenKind::Eof => break,
                _ => {
                    self.buffered = Some(token);
                    break;
                }
            }

            let next = self.peek_non_newline()?;
            match next.kind {
                YamlTokenKind::Dash => continue,
                YamlTokenKind::Dedent | YamlTokenKind::Eof => break,
                _ => break,
            }
        }

        self.depth = self.depth.saturating_sub(1);
        Ok(Value::Array(Array(items)))
    }

    fn parse_sequence_item(&mut self) -> Result<Value> {
        let token = self.next_token()?;
        match token.kind {
            YamlTokenKind::Newline => {
                let next = self.next_non_newline()?;
                match next.kind {
                    YamlTokenKind::Indent => {
                        let value = self.parse_block()?;
                        let end = self.next_non_newline()?;
                        if end.kind != YamlTokenKind::Dedent {
                            self.buffered = Some(end);
                        }
                        Ok(value)
                    }
                    YamlTokenKind::Dedent => {
                        self.buffered = Some(next);
                        Ok(Value::Null)
                    }
                    _ => {
                        self.buffered = Some(next);
                        Ok(Value::Null)
                    }
                }
            }
            YamlTokenKind::Scalar(value) => {
                let peek = self.peek_token()?;
                if peek.kind == YamlTokenKind::Colon {
                    let obj = self.parse_mapping_entries(Some(value))?;
                    Ok(Value::Object(obj))
                } else {
                    Ok(parse_scalar_value(&value))
                }
            }
            YamlTokenKind::LeftBracket => self.parse_flow_sequence(),
            YamlTokenKind::LeftBrace => self.parse_flow_mapping(),
            YamlTokenKind::Indent => {
                let value = self.parse_block()?;
                let end = self.next_non_newline()?;
                if end.kind != YamlTokenKind::Dedent {
                    self.buffered = Some(end);
                }
                Ok(value)
            }
            _ => Err(Error::with_message(
                ErrorKind::InvalidToken,
                token.span,
                "invalid sequence item".to_string(),
            )),
        }
    }

    fn parse_mapping(&mut self) -> Result<Value> {
        self.bump_depth()?;
        let obj = self.parse_mapping_entries(None)?;
        self.depth = self.depth.saturating_sub(1);
        Ok(Value::Object(obj))
    }

    fn parse_mapping_or_scalar(&mut self) -> Result<Value> {
        let first = self.next_non_newline()?;
        match first.kind {
            YamlTokenKind::Scalar(value) => {
                let next = self.peek_token()?;
                if next.kind == YamlTokenKind::Colon {
                    let obj = self.parse_mapping_entries(Some(value))?;
                    Ok(Value::Object(obj))
                } else {
                    Ok(parse_scalar_value(&value))
                }
            }
            _ => {
                self.buffered = Some(first);
                self.parse_mapping()
            }
        }
    }

    fn parse_mapping_entries(&mut self, first_key: Option<String>) -> Result<Object> {
        let mut obj = Object::new();
        let mut pending_key = first_key;

        loop {
            let key = if let Some(key) = pending_key.take() {
                key
            } else {
                let token = self.next_non_newline()?;
                match token.kind {
                    YamlTokenKind::Scalar(value) => value,
                    YamlTokenKind::Dedent | YamlTokenKind::Eof => {
                        self.buffered = Some(token);
                        break;
                    }
                    YamlTokenKind::Dash => {
                        self.buffered = Some(token);
                        break;
                    }
                    _ => {
                        return Err(Error::with_message(
                            ErrorKind::InvalidToken,
                            token.span,
                            "expected mapping key".to_string(),
                        ));
                    }
                }
            };

            let colon = self.next_non_newline()?;
            if colon.kind != YamlTokenKind::Colon {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    colon.span,
                    "expected ':'".to_string(),
                ));
            }

            let token = self.next_token()?;
            let value = match token.kind {
                YamlTokenKind::Scalar(value) => parse_scalar_value(&value),
                YamlTokenKind::Newline => {
                    let next = self.next_non_newline()?;
                    match next.kind {
                        YamlTokenKind::Indent => {
                            let value = self.parse_block()?;
                            let end = self.next_non_newline()?;
                            if end.kind != YamlTokenKind::Dedent {
                                self.buffered = Some(end);
                            }
                            value
                        }
                        YamlTokenKind::Dedent => {
                            self.buffered = Some(next);
                            Value::Null
                        }
                        _ => {
                            self.buffered = Some(next);
                            Value::Null
                        }
                    }
                }
                YamlTokenKind::Indent => {
                    let value = self.parse_block()?;
                    let end = self.next_non_newline()?;
                    if end.kind != YamlTokenKind::Dedent {
                        self.buffered = Some(end);
                    }
                    value
                }
                YamlTokenKind::LeftBracket => self.parse_flow_sequence()?,
                YamlTokenKind::LeftBrace => self.parse_flow_mapping()?,
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidToken,
                        token.span,
                        "expected value".to_string(),
                    ));
                }
            };

            if obj.contains_key(&key) {
                return Err(Error::with_message(
                    ErrorKind::DuplicateKey { key },
                    Span::empty(),
                    "duplicate key".to_string(),
                ));
            }
            obj.insert(&key, value);

            let next = self.peek_non_newline()?;
            match next.kind {
                YamlTokenKind::Scalar(_) => continue,
                YamlTokenKind::Dedent | YamlTokenKind::Eof => break,
                YamlTokenKind::Dash => break,
                _ => break,
            }
        }

        Ok(obj)
    }

    fn bump_depth(&mut self) -> Result<()> {
        self.depth = self.depth.saturating_add(1);
        if self.config.max_depth > 0 && self.depth > self.config.max_depth {
            return Err(Error::with_message(
                ErrorKind::MaxDepthExceeded {
                    max: self.config.max_depth,
                },
                Span::empty(),
                "max depth exceeded".to_string(),
            ));
        }
        Ok(())
    }

    fn parse_flow_sequence(&mut self) -> Result<Value> {
        self.bump_depth()?;
        let mut items = Vec::new();

        loop {
            let token = self.next_non_newline()?;
            match token.kind {
                YamlTokenKind::RightBracket => break,
                YamlTokenKind::Comma => continue,
                YamlTokenKind::LeftBracket => {
                    let value = self.parse_flow_sequence()?;
                    items.push(value);
                }
                YamlTokenKind::LeftBrace => {
                    let value = self.parse_flow_mapping()?;
                    items.push(value);
                }
                YamlTokenKind::Scalar(value) => {
                    items.push(parse_scalar_value(&value));
                }
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidToken,
                        token.span,
                        "invalid flow sequence".to_string(),
                    ));
                }
            }
        }

        self.depth = self.depth.saturating_sub(1);
        Ok(Value::Array(Array(items)))
    }

    fn parse_flow_mapping(&mut self) -> Result<Value> {
        self.bump_depth()?;
        let mut obj = Object::new();

        loop {
            let token = self.next_non_newline()?;
            match token.kind {
                YamlTokenKind::RightBrace => break,
                YamlTokenKind::Comma => continue,
                YamlTokenKind::Scalar(key) => {
                    let colon = self.next_non_newline()?;
                    if colon.kind != YamlTokenKind::Colon {
                        return Err(Error::with_message(
                            ErrorKind::InvalidToken,
                            colon.span,
                            "expected ':' in flow mapping".to_string(),
                        ));
                    }

                    let value_token = self.next_non_newline()?;
                    let value = match value_token.kind {
                        YamlTokenKind::Scalar(value) => parse_scalar_value(&value),
                        YamlTokenKind::LeftBracket => self.parse_flow_sequence()?,
                        YamlTokenKind::LeftBrace => self.parse_flow_mapping()?,
                        _ => {
                            return Err(Error::with_message(
                                ErrorKind::InvalidToken,
                                value_token.span,
                                "expected value in flow mapping".to_string(),
                            ));
                        }
                    };

                    insert_flow_value(&mut obj, &key, value)?;

                    let next = self.peek_non_newline()?;
                    match next.kind {
                        YamlTokenKind::Comma => {
                            let _ = self.next_non_newline()?;
                        }
                        YamlTokenKind::RightBrace => {
                            let _ = self.next_non_newline()?;
                            break;
                        }
                        _ => {}
                    }
                }
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidToken,
                        token.span,
                        "invalid flow mapping".to_string(),
                    ));
                }
            }
        }

        self.depth = self.depth.saturating_sub(1);
        Ok(Value::Object(obj))
    }
}

fn insert_flow_value(obj: &mut Object, key: &str, value: Value) -> Result<()> {
    if obj.contains_key(key) {
        return Err(Error::with_message(
            ErrorKind::DuplicateKey {
                key: key.to_string(),
            },
            Span::empty(),
            "duplicate key".to_string(),
        ));
    }
    obj.insert(key, value);
    Ok(())
}

fn parse_scalar_value(value: &str) -> Value {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Value::String(String::new());
    }

    match trimmed {
        "null" | "Null" | "NULL" | "~" => return Value::Null,
        "true" | "True" | "TRUE" => return Value::Bool(true),
        "false" | "False" | "FALSE" => return Value::Bool(false),
        _ => {}
    }

    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Value::from(int_val);
    }

    if let Ok(float_val) = trimmed.parse::<f64>() {
        return Value::Number(float_val);
    }

    Value::String(trimmed.to_string())
}

fn emit_events(value: &Value, events: &mut VecDeque<Event>) {
    match value {
        Value::Object(obj) => {
            events.push_back(Event::MappingStart);
            for (key, value) in obj.iter() {
                events.push_back(Event::Key(key.clone()));
                emit_events(value, events);
            }
            events.push_back(Event::MappingEnd);
        }
        Value::Array(arr) => {
            events.push_back(Event::SequenceStart);
            for value in arr.iter() {
                emit_events(value, events);
            }
            events.push_back(Event::SequenceEnd);
        }
        _ => {
            events.push_back(Event::Value(value.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<()> {
        if left == right {
            Ok(())
        } else {
            Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                format!("assertion failed: left={left:?} right={right:?}"),
            ))
        }
    }

    #[test]
    fn test_parse_simple_mapping() -> Result<()> {
        let input = b"name: John\nage: 30\n";
        let mut parser = Parser::new(input);
        let value = parser.parse()?;

        if let Value::Object(obj) = value {
            ensure_eq(obj.get("name"), Some(&Value::String("John".to_string())))?;
            ensure_eq(obj.get("age"), Some(&Value::Number(30.0)))?;
        } else {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "expected object".to_string(),
            ));
        }
        Ok(())
    }

    #[test]
    fn test_parse_sequence() -> Result<()> {
        let input = b"- one\n- two\n";
        let mut parser = Parser::new(input);
        let value = parser.parse()?;

        if let Value::Array(arr) = value {
            ensure_eq(arr.len(), 2)?;
            ensure_eq(arr.get(0), Some(&Value::String("one".to_string())))?;
            ensure_eq(arr.get(1), Some(&Value::String("two".to_string())))?;
        } else {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "expected array".to_string(),
            ));
        }
        Ok(())
    }
}
