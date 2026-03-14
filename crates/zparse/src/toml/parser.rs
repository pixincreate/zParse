//! TOML streaming parser implementation

use std::collections::VecDeque;

use crate::error::{Error, ErrorKind, Result, Span};
use crate::lexer::toml::{TomlLexer, TomlToken, TomlTokenKind};
use crate::toml::event::Event;

pub const DEFAULT_MAX_DEPTH: u16 = 128;
pub const DEFAULT_MAX_SIZE: usize = 10 * 1024 * 1024;
use crate::value::{Array, Object, TomlDatetime, Value};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

/// Configuration for the TOML parser
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
            max_depth: DEFAULT_MAX_DEPTH,
            max_size: DEFAULT_MAX_SIZE,
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

/// Streaming TOML parser with depth and size limits
#[derive(Debug)]
pub struct Parser<'a> {
    lexer: TomlLexer<'a>,
    config: Config,
    bytes_parsed: usize,
    depth: u16,
    buffered: Option<TomlToken>,
    events: VecDeque<Event>,
    root: Object,
    current_table: Vec<String>,
    current_is_array: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser with default configuration
    pub fn new(input: &'a [u8]) -> Self {
        Self::with_config(input, Config::default())
    }

    /// Create a new parser with custom configuration
    pub fn with_config(input: &'a [u8], config: Config) -> Self {
        Self {
            lexer: TomlLexer::new(input),
            config,
            bytes_parsed: 0,
            depth: 0,
            buffered: None,
            events: VecDeque::new(),
            root: Object::new(),
            current_table: Vec::new(),
            current_is_array: false,
        }
    }

    /// Get the next event from the parser
    pub fn next_event(&mut self) -> Result<Option<Event>> {
        if let Some(event) = self.events.pop_front() {
            return Ok(Some(event));
        }

        let token = match self.next_non_newline_token()? {
            Some(token) => token,
            None => return Ok(None),
        };

        match token.kind {
            TomlTokenKind::LeftBracket | TomlTokenKind::DoubleLeftBracket => {
                let is_array = matches!(token.kind, TomlTokenKind::DoubleLeftBracket);
                let path = self.parse_table_header(token.kind)?;
                if is_array {
                    self.ensure_array_table(&path)?;
                } else {
                    self.ensure_table(&path)?;
                }
                self.current_table = path.clone();
                self.current_is_array = is_array;
                let event = Event::TableStart { path, is_array };
                Ok(Some(event))
            }
            _ => {
                let key = self.parse_key_path(Some(token))?;
                self.expect_kind(TomlTokenKind::Equals)?;
                let value = self.parse_value()?;
                let table_path = self.current_table.clone();
                let is_array = self.current_is_array;
                self.insert_dotted_key(&table_path, is_array, &key, value.clone())?;
                Ok(Some(Event::KeyValue { key, value }))
            }
        }
    }

    /// Parse the full document into a Value
    pub fn parse(&mut self) -> Result<Value> {
        while let Some(_event) = self.next_event()? {}
        Ok(Value::Object(std::mem::take(&mut self.root)))
    }

    fn next_token(&mut self) -> Result<TomlToken> {
        let token = match self.buffered.take() {
            Some(token) => token,
            None => self.lexer.next_token()?,
        };

        let span = token.span;
        self.bytes_parsed = span.end.offset;

        if self.config.max_size > 0 && self.bytes_parsed > self.config.max_size {
            return Err(Error::at(
                ErrorKind::MaxSizeExceeded {
                    max: self.config.max_size,
                },
                self.bytes_parsed,
                span.end.line,
                span.end.col,
            ));
        }

        Ok(token)
    }

    fn peek_token(&mut self) -> Result<TomlToken> {
        if self.buffered.is_none() {
            let token = self.lexer.next_token()?;
            self.buffered = Some(token);
        }
        self.buffered.clone().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "missing buffered token".to_string(),
            )
        })
    }

    fn next_non_newline_token(&mut self) -> Result<Option<TomlToken>> {
        loop {
            let token = self.next_token()?;
            match token.kind {
                TomlTokenKind::Newline => continue,
                TomlTokenKind::Eof => return Ok(None),
                _ => return Ok(Some(token)),
            }
        }
    }

    fn expect_kind(&mut self, expected: TomlTokenKind) -> Result<()> {
        let token = self.next_token()?;
        if token.kind == expected {
            Ok(())
        } else {
            Err(Error::with_message(
                ErrorKind::Expected {
                    expected: format!("{expected:?}"),
                    found: format!("{found:?}", found = token.kind),
                },
                token.span,
                "unexpected token".to_string(),
            ))
        }
    }

    fn parse_table_header(&mut self, kind: TomlTokenKind) -> Result<Vec<String>> {
        let close = match kind {
            TomlTokenKind::LeftBracket => TomlTokenKind::RightBracket,
            TomlTokenKind::DoubleLeftBracket => TomlTokenKind::DoubleRightBracket,
            _ => TomlTokenKind::RightBracket,
        };

        let mut path = Vec::new();
        let first = self.next_token()?;
        path.push(self.parse_key_from_token(first)?);

        loop {
            let token = self.next_token()?;
            match token.kind {
                TomlTokenKind::Dot => {
                    let next = self.next_token()?;
                    path.push(self.parse_key_from_token(next)?);
                }
                kind if kind == close => break,
                TomlTokenKind::Newline => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidKey,
                        token.span,
                        "newline not allowed in table header".to_string(),
                    ));
                }
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidKey,
                        token.span,
                        "invalid table header".to_string(),
                    ));
                }
            }
        }

        Ok(path)
    }

    fn parse_key_from_token(&self, token: TomlToken) -> Result<String> {
        match token.kind {
            TomlTokenKind::BareKey(key) => Ok(key),
            TomlTokenKind::String(key) => Ok(key),
            _ => Err(Error::with_message(
                ErrorKind::InvalidKey,
                token.span,
                "invalid key".to_string(),
            )),
        }
    }

    fn parse_key_path(&mut self, first: Option<TomlToken>) -> Result<Vec<String>> {
        let first = match first {
            Some(token) => token,
            None => self.next_token()?,
        };
        let mut path = vec![self.parse_key_from_token(first)?];

        loop {
            let token = self.peek_token()?;
            if token.kind != TomlTokenKind::Dot {
                break;
            }
            let _ = self.next_token()?;
            let next = self.next_token()?;
            path.push(self.parse_key_from_token(next)?);
        }

        Ok(path)
    }

    fn parse_value(&mut self) -> Result<Value> {
        let token = self.next_token()?;
        self.parse_value_from_token(token)
    }

    fn parse_value_from_token(&mut self, token: TomlToken) -> Result<Value> {
        let token = self.normalize_value_token(token)?;
        match token.kind {
            TomlTokenKind::String(value) => Ok(Value::String(value)),
            TomlTokenKind::Integer(value) => Ok(Value::from(value)),
            TomlTokenKind::Float(value) => Ok(Value::Number(value)),
            TomlTokenKind::Bool(value) => Ok(Value::Bool(value)),
            TomlTokenKind::Datetime(value) => {
                let datetime = parse_toml_datetime(&value)?;
                Ok(Value::Datetime(datetime))
            }
            TomlTokenKind::LeftBracket => self.parse_array(token.span),
            TomlTokenKind::LeftBrace => self.parse_inline_table(token.span),
            _ => Err(Error::with_message(
                ErrorKind::InvalidToken,
                token.span,
                "expected value".to_string(),
            )),
        }
    }

    fn parse_array(&mut self, opening_span: Span) -> Result<Value> {
        self.depth = self.depth.saturating_add(1);
        if self.config.max_depth > 0 && self.depth > self.config.max_depth {
            return Err(Error::with_message(
                ErrorKind::MaxDepthExceeded {
                    max: self.config.max_depth,
                },
                opening_span,
                "max depth exceeded".to_string(),
            ));
        }

        let mut values = Vec::new();

        loop {
            let token = self.next_non_newline_token()?;
            match token {
                None => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidArray,
                        Span::empty(),
                        "unterminated array".to_string(),
                    ));
                }
                Some(token) if token.kind == TomlTokenKind::RightBracket => break,
                Some(token) if token.kind == TomlTokenKind::DoubleRightBracket => {
                    self.buffered = Some(TomlToken::new(TomlTokenKind::RightBracket, token.span));
                    break;
                }
                Some(token) => {
                    let token = self.normalize_value_token(token)?;
                    let value = self.parse_value_from_token(token)?;
                    values.push(value);

                    let token = self.next_non_newline_token()?;
                    match token {
                        Some(token) if token.kind == TomlTokenKind::Comma => {
                            let next = self.next_non_newline_token()?;
                            match next {
                                Some(token) if token.kind == TomlTokenKind::RightBracket => {
                                    break;
                                }
                                Some(token) if token.kind == TomlTokenKind::DoubleRightBracket => {
                                    self.buffered = Some(TomlToken::new(
                                        TomlTokenKind::RightBracket,
                                        token.span,
                                    ));
                                    break;
                                }
                                Some(token) => {
                                    self.buffered = Some(token);
                                    continue;
                                }
                                None => {
                                    return Err(Error::with_message(
                                        ErrorKind::InvalidArray,
                                        Span::empty(),
                                        "unterminated array".to_string(),
                                    ));
                                }
                            }
                        }
                        Some(token) if token.kind == TomlTokenKind::RightBracket => break,
                        Some(token) if token.kind == TomlTokenKind::DoubleRightBracket => {
                            self.buffered =
                                Some(TomlToken::new(TomlTokenKind::RightBracket, token.span));
                            break;
                        }
                        Some(token) => {
                            return Err(Error::with_message(
                                ErrorKind::InvalidArray,
                                token.span,
                                "expected comma or closing bracket".to_string(),
                            ));
                        }
                        None => {
                            return Err(Error::with_message(
                                ErrorKind::InvalidArray,
                                Span::empty(),
                                "unterminated array".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        self.depth = self.depth.saturating_sub(1);
        Ok(Value::Array(Array(values)))
    }

    fn parse_inline_table(&mut self, opening_span: Span) -> Result<Value> {
        self.depth = self.depth.saturating_add(1);
        if self.config.max_depth > 0 && self.depth > self.config.max_depth {
            return Err(Error::with_message(
                ErrorKind::MaxDepthExceeded {
                    max: self.config.max_depth,
                },
                opening_span,
                "max depth exceeded".to_string(),
            ));
        }

        let mut obj = Object::new();

        let token = self.next_non_newline_token()?;
        match token {
            Some(token) if token.kind == TomlTokenKind::RightBrace => {
                self.depth = self.depth.saturating_sub(1);
                return Ok(Value::Object(obj));
            }
            Some(token) => {
                self.buffered = Some(token);
            }
            None => {
                return Err(Error::with_message(
                    ErrorKind::InvalidInlineTable,
                    Span::empty(),
                    "unterminated inline table".to_string(),
                ));
            }
        }

        loop {
            let key = self.parse_key_path(None)?;
            self.expect_kind(TomlTokenKind::Equals)?;
            let value = self.parse_value()?;
            insert_dotted_key_into(&mut obj, &key, value)?;

            let token = self.next_token()?;
            match token.kind {
                TomlTokenKind::Comma => {
                    let next = self.peek_token()?;
                    if next.kind == TomlTokenKind::RightBrace {
                        let _ = self.next_token()?;
                        break;
                    }
                }
                TomlTokenKind::RightBrace => break,
                TomlTokenKind::Newline => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidInlineTable,
                        token.span,
                        "newline not allowed in inline table".to_string(),
                    ));
                }
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidInlineTable,
                        token.span,
                        "expected comma or closing brace".to_string(),
                    ));
                }
            }
        }

        self.depth = self.depth.saturating_sub(1);
        Ok(Value::Object(obj))
    }

    fn normalize_value_token(&mut self, token: TomlToken) -> Result<TomlToken> {
        match token.kind {
            TomlTokenKind::DoubleLeftBracket => {
                self.buffered = Some(TomlToken::new(TomlTokenKind::LeftBracket, token.span));
                Ok(TomlToken::new(TomlTokenKind::LeftBracket, token.span))
            }
            TomlTokenKind::DoubleRightBracket => {
                self.buffered = Some(TomlToken::new(TomlTokenKind::RightBracket, token.span));
                Ok(TomlToken::new(TomlTokenKind::RightBracket, token.span))
            }
            _ => Ok(token),
        }
    }

    fn ensure_table(&mut self, path: &[String]) -> Result<()> {
        let _ = ensure_table_path(&mut self.root, path)?;
        Ok(())
    }

    fn ensure_array_table(&mut self, path: &[String]) -> Result<()> {
        let _ = ensure_array_table_path(&mut self.root, path)?;
        Ok(())
    }

    fn insert_dotted_key(
        &mut self,
        table_path: &[String],
        is_array: bool,
        key: &[String],
        value: Value,
    ) -> Result<()> {
        if is_array {
            let table = get_array_table_last(&mut self.root, table_path)?;
            insert_dotted_key_into(table, key, value)
        } else {
            let table = ensure_table_path(&mut self.root, table_path)?;
            insert_dotted_key_into(table, key, value)
        }
    }
}

fn parse_toml_datetime(value: &str) -> Result<TomlDatetime> {
    if let Ok(datetime) = OffsetDateTime::parse(value, &Rfc3339) {
        return Ok(TomlDatetime::OffsetDateTime(datetime));
    }

    let local_datetime = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    let local_datetime_frac =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]");
    let local_datetime_space = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let local_datetime_space_frac =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

    if let Ok(datetime) = PrimitiveDateTime::parse(value, &local_datetime) {
        return Ok(TomlDatetime::LocalDateTime(datetime));
    }
    if let Ok(datetime) = PrimitiveDateTime::parse(value, &local_datetime_frac) {
        return Ok(TomlDatetime::LocalDateTime(datetime));
    }
    if let Ok(datetime) = PrimitiveDateTime::parse(value, &local_datetime_space) {
        return Ok(TomlDatetime::LocalDateTime(datetime));
    }
    if let Ok(datetime) = PrimitiveDateTime::parse(value, &local_datetime_space_frac) {
        return Ok(TomlDatetime::LocalDateTime(datetime));
    }

    let local_date = format_description!("[year]-[month]-[day]");
    if let Ok(date) = Date::parse(value, &local_date) {
        return Ok(TomlDatetime::LocalDate(date));
    }

    let local_time = format_description!("[hour]:[minute]:[second]");
    let local_time_frac = format_description!("[hour]:[minute]:[second].[subsecond]");
    if let Ok(time) = Time::parse(value, &local_time) {
        return Ok(TomlDatetime::LocalTime(time));
    }
    if let Ok(time) = Time::parse(value, &local_time_frac) {
        return Ok(TomlDatetime::LocalTime(time));
    }

    Err(Error::with_message(
        ErrorKind::InvalidDatetime,
        Span::empty(),
        "invalid datetime".to_string(),
    ))
}

fn ensure_table_path<'a>(root: &'a mut Object, path: &[String]) -> Result<&'a mut Object> {
    let mut current = root;
    for part in path {
        // Check type without cloning by matching on immutable reference first
        match current.get(part) {
            Some(Value::Object(_)) => {}
            Some(Value::Array(_)) => {
                return Err(Error::with_message(
                    ErrorKind::InvalidArray,
                    Span::empty(),
                    "array used where table expected".to_string(),
                ));
            }
            Some(_) => {
                return Err(Error::with_message(
                    ErrorKind::InvalidKey,
                    Span::empty(),
                    "key already assigned".to_string(),
                ));
            }
            None => {
                current.insert(part, Object::new());
            }
        }
        // Now get mutable reference (safe because we checked type above)
        current = current
            .get_mut(part)
            .and_then(|value| match value {
                Value::Object(obj) => Some(obj),
                _ => None,
            })
            .ok_or_else(|| {
                Error::with_message(
                    ErrorKind::InvalidKey,
                    Span::empty(),
                    "expected table".to_string(),
                )
            })?;
    }
    Ok(current)
}

fn ensure_array_table_path<'a>(root: &'a mut Object, path: &[String]) -> Result<&'a mut Object> {
    if path.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidKey,
            Span::empty(),
            "empty array table path".to_string(),
        ));
    }

    let mut current = root;
    for (index, part) in path.iter().enumerate() {
        let is_last = index + 1 == path.len();

        if is_last {
            // Check existing type without cloning
            match current.get(part) {
                None => {
                    let mut array = Array::new();
                    array.push(Object::new());
                    current.insert(part, Value::Array(array));
                }
                Some(Value::Array(_)) => {
                    // Get mutable reference to push new entry
                    if let Some(Value::Array(array)) = current.get_mut(part) {
                        array.push(Object::new());
                    }
                }
                Some(_) => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidArray,
                        Span::empty(),
                        "array table conflicts with existing value".to_string(),
                    ));
                }
            }

            let array = current
                .get_mut(part)
                .and_then(|value| match value {
                    Value::Array(array) => Some(array),
                    _ => None,
                })
                .ok_or_else(|| {
                    Error::with_message(
                        ErrorKind::InvalidArray,
                        Span::empty(),
                        "expected array table".to_string(),
                    )
                })?;
            let last = array
                .iter_mut()
                .last()
                .and_then(|value| match value {
                    Value::Object(obj) => Some(obj),
                    _ => None,
                })
                .ok_or_else(|| {
                    Error::with_message(
                        ErrorKind::InvalidArray,
                        Span::empty(),
                        "expected object in array table".to_string(),
                    )
                })?;
            return Ok(last);
        }

        // Check type without cloning
        match current.get(part) {
            Some(Value::Object(_)) => {
                current = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidKey,
                            Span::empty(),
                            "expected table".to_string(),
                        )
                    })?;
            }
            Some(Value::Array(_)) => {
                let array = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Array(array) => Some(array),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidArray,
                            Span::empty(),
                            "expected array table".to_string(),
                        )
                    })?;
                let last = array
                    .iter_mut()
                    .last()
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidArray,
                            Span::empty(),
                            "expected object in array table".to_string(),
                        )
                    })?;
                current = last;
            }
            Some(_) => {
                return Err(Error::with_message(
                    ErrorKind::InvalidKey,
                    Span::empty(),
                    "key already assigned".to_string(),
                ));
            }
            None => {
                current.insert(part, Object::new());
                current = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidKey,
                            Span::empty(),
                            "expected table".to_string(),
                        )
                    })?;
            }
        }
    }

    Err(Error::with_message(
        ErrorKind::InvalidKey,
        Span::empty(),
        "invalid array table path".to_string(),
    ))
}

fn get_array_table_last<'a>(root: &'a mut Object, path: &[String]) -> Result<&'a mut Object> {
    if path.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidKey,
            Span::empty(),
            "empty array table path".to_string(),
        ));
    }

    let mut current = root;
    for (index, part) in path.iter().enumerate() {
        let is_last = index + 1 == path.len();

        if is_last {
            let array = current
                .get_mut(part)
                .and_then(|value| match value {
                    Value::Array(array) => Some(array),
                    _ => None,
                })
                .ok_or_else(|| {
                    Error::with_message(
                        ErrorKind::InvalidArray,
                        Span::empty(),
                        "expected array table".to_string(),
                    )
                })?;
            let last = array
                .iter_mut()
                .last()
                .and_then(|value| match value {
                    Value::Object(obj) => Some(obj),
                    _ => None,
                })
                .ok_or_else(|| {
                    Error::with_message(
                        ErrorKind::InvalidArray,
                        Span::empty(),
                        "expected object in array table".to_string(),
                    )
                })?;
            return Ok(last);
        }

        // Check type without cloning
        match current.get(part) {
            Some(Value::Object(_)) => {
                current = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidKey,
                            Span::empty(),
                            "expected table".to_string(),
                        )
                    })?;
            }
            Some(Value::Array(_)) => {
                let array = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Array(array) => Some(array),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidArray,
                            Span::empty(),
                            "expected array table".to_string(),
                        )
                    })?;
                let last = array
                    .iter_mut()
                    .last()
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidArray,
                            Span::empty(),
                            "expected object in array table".to_string(),
                        )
                    })?;
                current = last;
            }
            Some(_) => {
                return Err(Error::with_message(
                    ErrorKind::InvalidKey,
                    Span::empty(),
                    "key already assigned".to_string(),
                ));
            }
            None => {
                return Err(Error::with_message(
                    ErrorKind::InvalidKey,
                    Span::empty(),
                    "missing array table".to_string(),
                ));
            }
        }
    }

    Err(Error::with_message(
        ErrorKind::InvalidKey,
        Span::empty(),
        "invalid array table path".to_string(),
    ))
}

fn insert_dotted_key_into(table: &mut Object, key: &[String], value: Value) -> Result<()> {
    if key.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidKey,
            Span::empty(),
            "empty key".to_string(),
        ));
    }

    let mut current = table;
    let parts = key.get(..key.len().saturating_sub(1)).ok_or_else(|| {
        Error::with_message(
            ErrorKind::InvalidKey,
            Span::empty(),
            "empty key".to_string(),
        )
    })?;

    for part in parts {
        match current.get(part) {
            Some(Value::Object(_)) => {
                current = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidKey,
                            Span::empty(),
                            "expected table".to_string(),
                        )
                    })?;
            }
            Some(_) => {
                return Err(Error::with_message(
                    ErrorKind::InvalidKey,
                    Span::empty(),
                    "key already assigned".to_string(),
                ));
            }
            None => {
                current.insert(part, Object::new());
                current = current
                    .get_mut(part)
                    .and_then(|value| match value {
                        Value::Object(obj) => Some(obj),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidKey,
                            Span::empty(),
                            "expected table".to_string(),
                        )
                    })?;
            }
        }
    }

    let last = key.last().ok_or_else(|| {
        Error::with_message(
            ErrorKind::InvalidKey,
            Span::empty(),
            "empty key".to_string(),
        )
    })?;
    if current.contains_key(last) {
        return Err(Error::with_message(
            ErrorKind::DuplicateKey { key: last.clone() },
            Span::empty(),
            "duplicate key".to_string(),
        ));
    }
    current.insert(last, value);
    Ok(())
}
