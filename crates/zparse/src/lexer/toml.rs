//! TOML-specific lexer

use crate::error::{Error, ErrorKind, Result, Span};
use crate::lexer::cursor::Cursor;

/// TOML token types
#[derive(Clone, Debug, PartialEq)]
pub enum TomlTokenKind {
    LeftBracket,
    RightBracket,
    DoubleLeftBracket,
    DoubleRightBracket,
    LeftBrace,
    RightBrace,
    Equals,
    Comma,
    Dot,
    Newline,
    BareKey(String),
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Datetime(String),
    Eof,
}

/// TOML token with span information
#[derive(Clone, Debug, PartialEq)]
pub struct TomlToken {
    pub kind: TomlTokenKind,
    pub span: Span,
}

impl TomlToken {
    pub const fn new(kind: TomlTokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// TOML lexer
#[derive(Clone, Debug)]
pub struct TomlLexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> TomlLexer<'a> {
    /// Create a new TOML lexer from input bytes
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<TomlToken> {
        self.skip_space();

        let start = self.cursor.position();

        let kind = match self.cursor.current() {
            None => TomlTokenKind::Eof,
            Some(b'\n') => {
                self.cursor.advance();
                TomlTokenKind::Newline
            }
            Some(b'#') => {
                self.skip_comment();
                return self.next_token();
            }
            Some(b'[') => {
                if self.cursor.peek(1) == Some(b'[') {
                    self.cursor.advance_by(2);
                    TomlTokenKind::DoubleLeftBracket
                } else {
                    self.cursor.advance();
                    TomlTokenKind::LeftBracket
                }
            }
            Some(b']') => {
                if self.cursor.peek(1) == Some(b']') {
                    self.cursor.advance_by(2);
                    TomlTokenKind::DoubleRightBracket
                } else {
                    self.cursor.advance();
                    TomlTokenKind::RightBracket
                }
            }
            Some(b'{') => {
                self.cursor.advance();
                TomlTokenKind::LeftBrace
            }
            Some(b'}') => {
                self.cursor.advance();
                TomlTokenKind::RightBrace
            }
            Some(b'=') => {
                self.cursor.advance();
                TomlTokenKind::Equals
            }
            Some(b',') => {
                self.cursor.advance();
                TomlTokenKind::Comma
            }
            Some(b'.') => {
                self.cursor.advance();
                TomlTokenKind::Dot
            }
            Some(b'"') => self.lex_basic_string()?,
            Some(b'\'') => self.lex_literal_string()?,
            Some(b'-') => {
                if matches!(
                    self.cursor.peek(1),
                    Some(b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-')
                ) {
                    self.lex_bare_key_or_bool()?
                } else {
                    self.lex_number_or_datetime()?
                }
            }
            Some(b'+' | b'0'..=b'9') => self.lex_number_or_datetime()?,
            Some(b'A'..=b'Z' | b'a'..=b'z' | b'_') => self.lex_bare_key_or_bool()?,
            Some(_b) => {
                return Err(Error::at(
                    ErrorKind::InvalidToken,
                    start.offset,
                    start.line,
                    start.col,
                ));
            }
        };

        let end = self.cursor.position();
        Ok(TomlToken::new(kind, Span::new(start, end)))
    }

    fn skip_space(&mut self) {
        while let Some(b) = self.cursor.current() {
            if matches!(b, b' ' | b'\t' | b'\r') {
                self.cursor.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(b) = self.cursor.current() {
            self.cursor.advance();
            if b == b'\n' {
                break;
            }
        }
    }

    fn lex_basic_string(&mut self) -> Result<TomlTokenKind> {
        if self.cursor.peek_bytes(3) == Some(b"\"\"\"") {
            return self.lex_multiline_basic_string();
        }

        self.cursor.advance();
        let mut result = String::new();

        loop {
            match self.cursor.current() {
                None => {
                    return Err(Error::at(
                        ErrorKind::UnterminatedString,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
                Some(b'"') => {
                    self.cursor.advance();
                    break;
                }
                Some(b'\'') => {
                    self.cursor.advance();
                    result.push(self.lex_basic_escape()?);
                }
                Some(b'\n') => {
                    return Err(Error::at(
                        ErrorKind::UnterminatedString,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
                Some(b) => {
                    result.push(char::from(b));
                    self.cursor.advance();
                }
            }
        }

        Ok(TomlTokenKind::String(result))
    }

    fn lex_multiline_basic_string(&mut self) -> Result<TomlTokenKind> {
        self.cursor.advance_by(3);
        let mut result = String::new();

        loop {
            match self.cursor.current() {
                None => {
                    return Err(Error::at(
                        ErrorKind::UnterminatedString,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
                Some(b'"') => {
                    if self.cursor.peek_bytes(3) == Some(b"\"\"\"") {
                        self.cursor.advance_by(3);
                        break;
                    }
                    result.push('"');
                    self.cursor.advance();
                }
                Some(b'\\') => {
                    self.cursor.advance();
                    result.push(self.lex_basic_escape()?);
                }
                Some(b) => {
                    result.push(char::from(b));
                    self.cursor.advance();
                }
            }
        }

        Ok(TomlTokenKind::String(result))
    }

    fn lex_basic_escape(&mut self) -> Result<char> {
        match self.cursor.current() {
            Some(b'"') => {
                self.cursor.advance();
                Ok('"')
            }
            Some(b'\\') => {
                self.cursor.advance();
                Ok('\\')
            }
            Some(b'n') => {
                self.cursor.advance();
                Ok('\n')
            }
            Some(b'r') => {
                self.cursor.advance();
                Ok('\r')
            }
            Some(b't') => {
                self.cursor.advance();
                Ok('\t')
            }
            Some(b'b') => {
                self.cursor.advance();
                Ok('\x08')
            }
            Some(b'f') => {
                self.cursor.advance();
                Ok('\x0C')
            }
            Some(b'u') => {
                self.cursor.advance();
                self.lex_unicode_escape(4)
            }
            Some(b'U') => {
                self.cursor.advance();
                self.lex_unicode_escape(8)
            }
            _ => Err(Error::at(
                ErrorKind::InvalidEscapeSequence,
                self.cursor.position().offset,
                self.cursor.position().line,
                self.cursor.position().col,
            )),
        }
    }

    fn lex_unicode_escape(&mut self, digits: usize) -> Result<char> {
        let mut value: u32 = 0;
        for _ in 0..digits {
            let b = match self.cursor.current() {
                Some(b) => b,
                None => {
                    return Err(Error::at(
                        ErrorKind::InvalidUnicodeEscape,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
            };
            let digit = match b {
                b'0'..=b'9' => u32::from(b - b'0'),
                b'a'..=b'f' => u32::from(b - b'a') + 10,
                b'A'..=b'F' => u32::from(b - b'A') + 10,
                _ => {
                    return Err(Error::at(
                        ErrorKind::InvalidUnicodeEscape,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
            };
            value = value.saturating_mul(16).saturating_add(digit);
            self.cursor.advance();
        }
        match char::from_u32(value) {
            Some(ch) => Ok(ch),
            None => Err(Error::at(
                ErrorKind::InvalidUnicodeEscape,
                self.cursor.position().offset,
                self.cursor.position().line,
                self.cursor.position().col,
            )),
        }
    }

    fn lex_literal_string(&mut self) -> Result<TomlTokenKind> {
        if self.cursor.peek_bytes(3) == Some(b"'''") {
            return self.lex_multiline_literal_string();
        }

        self.cursor.advance();
        let mut result = String::new();

        loop {
            match self.cursor.current() {
                None => {
                    return Err(Error::at(
                        ErrorKind::UnterminatedString,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
                Some(b'\'') => {
                    self.cursor.advance();
                    break;
                }
                Some(b'\n') => {
                    return Err(Error::at(
                        ErrorKind::UnterminatedString,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
                Some(b) => {
                    result.push(char::from(b));
                    self.cursor.advance();
                }
            }
        }

        Ok(TomlTokenKind::String(result))
    }

    fn lex_multiline_literal_string(&mut self) -> Result<TomlTokenKind> {
        self.cursor.advance_by(3);
        let mut result = String::new();

        loop {
            match self.cursor.current() {
                None => {
                    return Err(Error::at(
                        ErrorKind::UnterminatedString,
                        self.cursor.position().offset,
                        self.cursor.position().line,
                        self.cursor.position().col,
                    ));
                }
                Some(b'\'') => {
                    if self.cursor.peek_bytes(3) == Some(b"'''") {
                        self.cursor.advance_by(3);
                        break;
                    }
                    result.push('\'');
                    self.cursor.advance();
                }
                Some(b) => {
                    result.push(char::from(b));
                    self.cursor.advance();
                }
            }
        }

        Ok(TomlTokenKind::String(result))
    }

    fn lex_bare_key_or_bool(&mut self) -> Result<TomlTokenKind> {
        let start = self.cursor.pos();
        while let Some(b) = self.cursor.current() {
            if matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-') {
                self.cursor.advance();
            } else {
                break;
            }
        }

        let raw = self.cursor.slice_from(start);
        let text = std::str::from_utf8(raw).map_err(|_| {
            Error::at(
                ErrorKind::InvalidToken,
                self.cursor.position().offset,
                self.cursor.position().line,
                self.cursor.position().col,
            )
        })?;

        match text {
            "true" => Ok(TomlTokenKind::Bool(true)),
            "false" => Ok(TomlTokenKind::Bool(false)),
            _ => Ok(TomlTokenKind::BareKey(text.to_string())),
        }
    }

    fn lex_number_or_datetime(&mut self) -> Result<TomlTokenKind> {
        let start = self.cursor.pos();
        if self.cursor.current() == Some(b'+') || self.cursor.current() == Some(b'-') {
            self.cursor.advance();
        }

        while let Some(b) = self.cursor.current() {
            if matches!(
                b,
                b'0'..=b'9' | b'_' | b'.' | b'e' | b'E' | b':' | b'T' | b'Z' | b'z' | b'-' | b'+'
            ) {
                self.cursor.advance();
            } else {
                break;
            }
        }

        let raw = self.cursor.slice_from(start);
        let text = std::str::from_utf8(raw).map_err(|_| {
            Error::at(
                ErrorKind::InvalidToken,
                self.cursor.position().offset,
                self.cursor.position().line,
                self.cursor.position().col,
            )
        })?;

        if is_datetime_like(text) {
            return Ok(TomlTokenKind::Datetime(text.to_string()));
        }

        let normalized = text.replace('_', "");
        if let Some(float) = parse_special_float(&normalized) {
            return Ok(TomlTokenKind::Float(float));
        }
        if normalized.contains(['.', 'e', 'E']) {
            let value = normalized.parse::<f64>().map_err(|_| {
                Error::at(
                    ErrorKind::InvalidNumber,
                    self.cursor.position().offset,
                    self.cursor.position().line,
                    self.cursor.position().col,
                )
            })?;
            return Ok(TomlTokenKind::Float(value));
        }

        let (sign, digits) = match normalized.strip_prefix('-') {
            Some(rest) => (-1_i64, rest),
            None => match normalized.strip_prefix('+') {
                Some(rest) => (1_i64, rest),
                None => (1_i64, normalized.as_str()),
            },
        };

        let (radix, digits) = if let Some(rest) = digits.strip_prefix("0x") {
            (16_u32, rest)
        } else if let Some(rest) = digits.strip_prefix("0o") {
            (8_u32, rest)
        } else if let Some(rest) = digits.strip_prefix("0b") {
            (2_u32, rest)
        } else {
            (10_u32, digits)
        };

        let unsigned = i64::from_str_radix(digits, radix).map_err(|_| {
            Error::at(
                ErrorKind::InvalidNumber,
                self.cursor.position().offset,
                self.cursor.position().line,
                self.cursor.position().col,
            )
        })?;
        let value = unsigned.saturating_mul(sign);
        Ok(TomlTokenKind::Integer(value))
    }
}

fn parse_special_float(text: &str) -> Option<f64> {
    match text {
        "inf" | "+inf" => Some(f64::INFINITY),
        "-inf" => Some(f64::NEG_INFINITY),
        "nan" | "+nan" | "-nan" => Some(f64::NAN),
        _ => None,
    }
}

fn is_datetime_like(text: &str) -> bool {
    if text.contains('T') || text.contains(':') || text.ends_with('Z') || text.ends_with('z') {
        return true;
    }

    let mut dash_count = 0_u8;
    for ch in text.chars() {
        if ch == '-' {
            dash_count = dash_count.saturating_add(1);
        } else if !ch.is_ascii_digit() {
            return false;
        }
    }

    dash_count >= 2 && text.len() >= 8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    fn test_simple_tokens() -> Result<()> {
        let input = b"[table]\nkey = 1\n";
        let mut lexer = TomlLexer::new(input);

        matches_token(&mut lexer, TomlTokenKind::LeftBracket)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("table".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::RightBracket)?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("key".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::Integer(1))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        Ok(())
    }

    #[test]
    fn test_string_tokens() -> Result<()> {
        let input = b"title = \"hello\"\nname = 'world'\n";
        let mut lexer = TomlLexer::new(input);

        matches_token(&mut lexer, TomlTokenKind::BareKey("title".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::String("hello".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("name".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::String("world".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        Ok(())
    }

    #[test]
    fn test_numbers_and_bool() -> Result<()> {
        let input = b"flag = true\nint = -42\nfloat = 3.5\n";
        let mut lexer = TomlLexer::new(input);

        matches_token(&mut lexer, TomlTokenKind::BareKey("flag".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::Bool(true))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("int".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::Integer(-42))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("float".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::Float(3.5))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        Ok(())
    }

    #[test]
    fn test_array_table_tokens() -> Result<()> {
        let input = b"[[products]]\nname = \"book\"\n";
        let mut lexer = TomlLexer::new(input);

        matches_token(&mut lexer, TomlTokenKind::DoubleLeftBracket)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("products".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::DoubleRightBracket)?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        matches_token(&mut lexer, TomlTokenKind::BareKey("name".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(&mut lexer, TomlTokenKind::String("book".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        Ok(())
    }

    #[test]
    fn test_datetime_token() -> Result<()> {
        let input = b"date = 1979-05-27T07:32:00Z\n";
        let mut lexer = TomlLexer::new(input);

        matches_token(&mut lexer, TomlTokenKind::BareKey("date".to_string()))?;
        matches_token(&mut lexer, TomlTokenKind::Equals)?;
        matches_token(
            &mut lexer,
            TomlTokenKind::Datetime("1979-05-27T07:32:00Z".to_string()),
        )?;
        matches_token(&mut lexer, TomlTokenKind::Newline)?;
        Ok(())
    }

    fn matches_token(lexer: &mut TomlLexer<'_>, expected: TomlTokenKind) -> Result<()> {
        let token = lexer.next_token()?;
        if token.kind != expected {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                token.span,
                format!("expected {expected:?}, got {actual:?}", actual = token.kind),
            ));
        }
        Ok(())
    }
}
