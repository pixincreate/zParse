//! JSON-specific lexer

use crate::error::{Error, ErrorKind, Result, Span};
use crate::lexer::cursor::Cursor;
use crate::lexer::token::{Token, TokenKind};

/// JSON lexer that tokenizes JSON input
#[derive(Clone, Debug)]
pub struct JsonLexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> JsonLexer<'a> {
    /// Create a new JSON lexer from input bytes
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token> {
        self.cursor.skip_whitespace();

        let start = self.cursor.position();

        let kind = match self.cursor.current() {
            None => TokenKind::Eof,
            Some(b) => match b {
                b'{' => {
                    self.cursor.advance();
                    TokenKind::LeftBrace
                }
                b'}' => {
                    self.cursor.advance();
                    TokenKind::RightBrace
                }
                b'[' => {
                    self.cursor.advance();
                    TokenKind::LeftBracket
                }
                b']' => {
                    self.cursor.advance();
                    TokenKind::RightBracket
                }
                b':' => {
                    self.cursor.advance();
                    TokenKind::Colon
                }
                b',' => {
                    self.cursor.advance();
                    TokenKind::Comma
                }
                b'"' => self.lex_string()?,
                b'n' => self.lex_null()?,
                b't' => self.lex_true()?,
                b'f' => self.lex_false()?,
                b'-' | b'0'..=b'9' => self.lex_number()?,
                _ => {
                    return Err(Error::at(
                        ErrorKind::InvalidToken,
                        start.offset,
                        start.line,
                        start.col,
                    ));
                }
            },
        };

        let end = self.cursor.position();
        Ok(Token::new(kind, Span::new(start, end)))
    }

    /// Lex a string literal
    fn lex_string(&mut self) -> Result<TokenKind> {
        // Consume opening quote
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
                Some(b'\\') => {
                    self.cursor.advance();
                    match self.cursor.current() {
                        None => {
                            return Err(Error::at(
                                ErrorKind::InvalidEscapeSequence,
                                self.cursor.position().offset,
                                self.cursor.position().line,
                                self.cursor.position().col,
                            ));
                        }
                        Some(escape_char) => {
                            match escape_char {
                                b'"' => result.push('"'),
                                b'\\' => result.push('\\'),
                                b'/' => result.push('/'),
                                b'b' => result.push('\x08'),
                                b'f' => result.push('\x0C'),
                                b'n' => result.push('\n'),
                                b'r' => result.push('\r'),
                                b't' => result.push('\t'),
                                b'u' => {
                                    self.cursor.advance();
                                    let code_point = self.lex_unicode_escape()?;
                                    result.push(code_point);
                                    continue;
                                }
                                _ => {
                                    return Err(Error::at(
                                        ErrorKind::InvalidEscapeSequence,
                                        self.cursor.position().offset,
                                        self.cursor.position().line,
                                        self.cursor.position().col,
                                    ));
                                }
                            }
                            self.cursor.advance();
                        }
                    }
                }
                Some(b) => {
                    // JSON strings cannot contain control characters
                    if b < 0x20 {
                        return Err(Error::at(
                            ErrorKind::InvalidToken,
                            self.cursor.position().offset,
                            self.cursor.position().line,
                            self.cursor.position().col,
                        ));
                    }
                    result.push(b as char);
                    self.cursor.advance();
                }
            }
        }

        Ok(TokenKind::String(result))
    }

    /// Lex a unicode escape sequence (\uXXXX)
    fn lex_unicode_escape(&mut self) -> Result<char> {
        let start_pos = self.cursor.position();
        let mut code: u32 = 0;

        for _ in 0..4 {
            match self.cursor.current() {
                None => {
                    return Err(Error::at(
                        ErrorKind::InvalidUnicodeEscape,
                        start_pos.offset,
                        start_pos.line,
                        start_pos.col,
                    ));
                }
                Some(b) => {
                    let digit = match b {
                        b'0'..=b'9' => (b - b'0') as u32,
                        b'a'..=b'f' => (b - b'a' + 10) as u32,
                        b'A'..=b'F' => (b - b'A' + 10) as u32,
                        _ => {
                            return Err(Error::at(
                                ErrorKind::InvalidUnicodeEscape,
                                self.cursor.position().offset,
                                self.cursor.position().line,
                                self.cursor.position().col,
                            ));
                        }
                    };
                    code = code * 16 + digit;
                    self.cursor.advance();
                }
            }
        }

        char::from_u32(code).ok_or_else(|| {
            Error::at(
                ErrorKind::InvalidUnicodeEscape,
                start_pos.offset,
                start_pos.line,
                start_pos.col,
            )
        })
    }

    /// Lex null literal
    fn lex_null(&mut self) -> Result<TokenKind> {
        if self.cursor.peek_bytes(4) == Some(b"null") {
            self.cursor.advance_by(4);
            Ok(TokenKind::Null)
        } else {
            let pos = self.cursor.position();
            Err(Error::at(
                ErrorKind::InvalidToken,
                pos.offset,
                pos.line,
                pos.col,
            ))
        }
    }

    /// Lex true literal
    fn lex_true(&mut self) -> Result<TokenKind> {
        if self.cursor.peek_bytes(4) == Some(b"true") {
            self.cursor.advance_by(4);
            Ok(TokenKind::True)
        } else {
            let pos = self.cursor.position();
            Err(Error::at(
                ErrorKind::InvalidToken,
                pos.offset,
                pos.line,
                pos.col,
            ))
        }
    }

    /// Lex false literal
    fn lex_false(&mut self) -> Result<TokenKind> {
        if self.cursor.peek_bytes(5) == Some(b"false") {
            self.cursor.advance_by(5);
            Ok(TokenKind::False)
        } else {
            let pos = self.cursor.position();
            Err(Error::at(
                ErrorKind::InvalidToken,
                pos.offset,
                pos.line,
                pos.col,
            ))
        }
    }

    /// Lex a number literal
    fn lex_number(&mut self) -> Result<TokenKind> {
        let start = self.cursor.pos();

        // Optional minus sign
        if self.cursor.current() == Some(b'-') {
            self.cursor.advance();
        }

        // Integer part
        match self.cursor.current() {
            Some(b'0') => {
                self.cursor.advance();
            }
            Some(b'1'..=b'9') => {
                self.cursor.advance();
                while let Some(b'0'..=b'9') = self.cursor.current() {
                    self.cursor.advance();
                }
            }
            _ => {
                let pos = self.cursor.position();
                return Err(Error::at(
                    ErrorKind::InvalidNumber,
                    pos.offset,
                    pos.line,
                    pos.col,
                ));
            }
        }

        // Optional fraction part
        if self.cursor.current() == Some(b'.') {
            self.cursor.advance();
            let has_digits = matches!(self.cursor.current(), Some(b'0'..=b'9'));
            if !has_digits {
                let pos = self.cursor.position();
                return Err(Error::at(
                    ErrorKind::InvalidNumber,
                    pos.offset,
                    pos.line,
                    pos.col,
                ));
            }
            while let Some(b'0'..=b'9') = self.cursor.current() {
                self.cursor.advance();
            }
        }

        // Optional exponent part
        if matches!(self.cursor.current(), Some(b'e') | Some(b'E')) {
            self.cursor.advance();
            if matches!(self.cursor.current(), Some(b'+') | Some(b'-')) {
                self.cursor.advance();
            }
            let has_digits = matches!(self.cursor.current(), Some(b'0'..=b'9'));
            if !has_digits {
                let pos = self.cursor.position();
                return Err(Error::at(
                    ErrorKind::InvalidNumber,
                    pos.offset,
                    pos.line,
                    pos.col,
                ));
            }
            while let Some(b'0'..=b'9') = self.cursor.current() {
                self.cursor.advance();
            }
        }

        // Parse the number
        let num_str = std::str::from_utf8(self.cursor.slice_from(start)).map_err(|_| {
            let pos = self.cursor.position();
            Error::at(ErrorKind::InvalidNumber, pos.offset, pos.line, pos.col)
        })?;

        let num = num_str.parse::<f64>().map_err(|_| {
            let pos = self.cursor.position();
            Error::at(ErrorKind::InvalidNumber, pos.offset, pos.line, pos.col)
        })?;

        Ok(TokenKind::Number(num))
    }
}

impl<'a> Iterator for JsonLexer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(token) => {
                if token.kind == TokenKind::Eof {
                    None
                } else {
                    Some(Ok(token))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_structural_tokens() {
        let input = b"{ } [ ] : ,";
        let mut lexer = JsonLexer::new(input);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LeftBrace);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RightBrace);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LeftBracket);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RightBracket);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn test_lexer_literals() {
        let input = b"null true false";
        let mut lexer = JsonLexer::new(input);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Null);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::True);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::False);
    }

    #[test]
    fn test_lexer_string() {
        let input = br#""hello world""#;
        let mut lexer = JsonLexer::new(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String("hello world".to_string()));
    }

    #[test]
    fn test_lexer_string_escapes() {
        let input = br#""hello\nworld\t!\"\\\/\b\f""#;
        let mut lexer = JsonLexer::new(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::String("hello\nworld\t!\"\\/\x08\x0C".to_string())
        );
    }

    #[test]
    fn test_lexer_string_unicode_escape() {
        let input = br#""hello \u0041\u0042\u0043""#;
        let mut lexer = JsonLexer::new(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String("hello ABC".to_string()));
    }

    #[test]
    fn test_lexer_number_integer() {
        let input = b"123 -456 0";
        let mut lexer = JsonLexer::new(input);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(123.0));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(-456.0));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(0.0));
    }

    #[test]
    fn test_lexer_number_fraction() {
        let input = b"3.14 -0.5 123.456";
        let mut lexer = JsonLexer::new(input);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(3.14));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(-0.5));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(123.456));
    }

    #[test]
    fn test_lexer_number_exponent() {
        let input = b"1e10 1E10 1e+5 1e-5 3.14e-2";
        let mut lexer = JsonLexer::new(input);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1e10));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1E10));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1e5));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1e-5));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(3.14e-2));
    }

    #[test]
    fn test_lexer_iterator() {
        let input = b"[1, 2, 3]";
        let lexer = JsonLexer::new(input);
        let tokens: Vec<_> = lexer.map(|t| t.unwrap().kind).collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::LeftBracket,
                TokenKind::Number(1.0),
                TokenKind::Comma,
                TokenKind::Number(2.0),
                TokenKind::Comma,
                TokenKind::Number(3.0),
                TokenKind::RightBracket,
            ]
        );
    }

    #[test]
    fn test_lexer_unterminated_string() {
        let input = br#""hello"#;
        let mut lexer = JsonLexer::new(input);

        let result = lexer.next_token();
        assert!(result.is_err());
        assert_eq!(*result.unwrap_err().kind(), ErrorKind::UnterminatedString);
    }

    #[test]
    fn test_lexer_invalid_escape() {
        let input = br#""hello\x""#;
        let mut lexer = JsonLexer::new(input);

        let result = lexer.next_token();
        assert!(result.is_err());
        assert_eq!(
            *result.unwrap_err().kind(),
            ErrorKind::InvalidEscapeSequence
        );
    }

    #[test]
    fn test_lexer_invalid_number() {
        let input = b"01";
        let mut lexer = JsonLexer::new(input);

        // This should parse as 0, then 1 as separate token
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(0.0));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1.0));
    }

    #[test]
    fn test_lexer_invalid_token() {
        let input = b"@";
        let mut lexer = JsonLexer::new(input);

        let result = lexer.next_token();
        assert!(result.is_err());
        assert_eq!(*result.unwrap_err().kind(), ErrorKind::InvalidToken);
    }

    #[test]
    fn test_lexer_empty_string() {
        let input = b"\"\"";
        let mut lexer = JsonLexer::new(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String("".to_string()));
    }

    #[test]
    fn test_lexer_unicode_escape_invalid() {
        let input = br#""\u00GH""#;
        let mut lexer = JsonLexer::new(input);

        let result = lexer.next_token();
        assert!(result.is_err());
        assert_eq!(*result.unwrap_err().kind(), ErrorKind::InvalidUnicodeEscape);
    }

    #[test]
    fn test_lexer_whitespace() {
        let input = b"  \t\n\r  null  ";
        let mut lexer = JsonLexer::new(input);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Null);
        assert_eq!(token.span.start.line, 2);
    }
}
