use super::token::Token;
use crate::error::{ParseError, ParseErrorKind, Result};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    is_json_mode: bool,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        let current_char = input.first().copied();
        Self {
            input,
            position: 0,
            current_char,
            is_json_mode: false,
        }
    }

    pub fn new_json(input: &str) -> Self {
        let mut lexer = Self::new(input);
        lexer.is_json_mode = true;
        lexer
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    fn read_string(&mut self) -> Result<String> {
        let mut result = String::new();
        // Skip the opening quote
        self.advance();

        while let Some(c) = self.current_char {
            match c {
                '"' => {
                    self.advance(); // Skip closing quote
                    return Ok(result);
                }
                '\\' => {
                    self.advance();
                    match self.current_char {
                        Some(escape_char) => {
                            let escaped = match escape_char {
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                '\\' => '\\',
                                '"' => '"',
                                _ => return Err(ParseError::new(ParseErrorKind::InvalidEscape(escape_char))),
                            };
                            result.push(escaped);
                            self.advance();
                        }
                        None => return Err(ParseError::new(ParseErrorKind::UnexpectedEOF)),
                    }
                }
                _ => {
                    result.push(c);
                    self.advance();
                }
            }
        }
        Err(ParseError::new(ParseErrorKind::UnexpectedEOF))
    }

    fn read_number(&mut self) -> Result<f64> {
        let mut number_str = String::new();
        let mut is_float = false;
        let mut has_digits = false;
        let mut previous_char_was_underscore = false;

        // Handle negative numbers
        if self.current_char == Some('-') {
            number_str.push('-');
            self.advance();
        }

        // Handle main part of the number
        while let Some(c) = self.current_char {
            match c {
                '0'..='9' => {
                    number_str.push(c);
                    has_digits = true;
                    previous_char_was_underscore = false;
                    self.advance();
                }
                '_' => {
                    if !has_digits || previous_char_was_underscore {
                        return Err(ParseError::new(ParseErrorKind::InvalidNumber(number_str)));
                    }
                    // Skip underscore but remember we saw it
                    previous_char_was_underscore = true;
                    self.advance();
                }
                '.' => {
                    if is_float || previous_char_was_underscore {
                        return Err(ParseError::new(ParseErrorKind::InvalidNumber(number_str)));
                    }
                    is_float = true;
                    previous_char_was_underscore = false;
                    number_str.push(c);
                    self.advance();
                }
                _ => break,
            }
        }

        if !has_digits {
            return Err(ParseError::new(ParseErrorKind::InvalidNumber(number_str)));
        }

        // Remove underscores before parsing
        let clean_number_str: String = number_str.chars().filter(|&c| c != '_').collect();

        clean_number_str
            .parse::<f64>()
            .map_err(|_| ParseError::new(ParseErrorKind::InvalidNumber(number_str)))
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::EOF),
            Some(c) => match c {
                '{' => {
                    self.advance();
                    Ok(Token::LeftBrace)
                }
                '}' => {
                    self.advance();
                    Ok(Token::RightBrace)
                }
                '[' => {
                    self.advance();
                    Ok(Token::LeftBracket)
                }
                ']' => {
                    self.advance();
                    Ok(Token::RightBracket)
                }
                ':' => {
                    self.advance();
                    Ok(Token::Colon)
                }
                ',' => {
                    self.advance();
                    Ok(Token::Comma)
                }
                '=' => {
                    self.advance();
                    Ok(Token::Equals)
                }
                '.' => {
                    self.advance();
                    Ok(Token::Dot)
                }
                '"' => {
                    let s = self.read_string()?;
                    Ok(Token::String(s))
                }
                '0'..='9' | '-' | '_' => {
                    let n = self.read_number()?;
                    Ok(Token::Number(n))
                }
                't' => self.read_true_or_identifier(),
                'f' => self.read_false_or_identifier(),
                'n' => self.read_null_or_identifier(),
                _ if Self::is_bare_key_start(c) => {
                    if self.is_json_mode {
                        Err(ParseError::new(ParseErrorKind::InvalidToken(format!(
                            "Unexpected character: {}. JSON strings must be quoted",
                            c
                        ))))
                    } else {
                        let s = self.read_bare_key()?;
                        Ok(Token::String(s))
                    }
                }
                _ => Err(ParseError::new(ParseErrorKind::InvalidToken(c.to_string()))),
            },
        }
    }

    fn is_bare_key_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_bare_key_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }

    fn read_bare_key(&mut self) -> Result<String> {
        let mut key = String::new();

        while let Some(c) = self.current_char {
            if Self::is_bare_key_char(c) {
                key.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if key.is_empty() {
            Err(ParseError::new(ParseErrorKind::InvalidKey("Empty key".to_string())))
        } else {
            Ok(key)
        }
    }

    fn read_true_or_identifier(&mut self) -> Result<Token> {
        let mut value = String::new();
        // Safely get the current character
        if let Some(c) = self.current_char {
            value.push(c);
        } else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }
        self.advance();

        // Try to read "true"
        if self.current_char == Some('r') {
            value.push('r');
            self.advance();
            if self.current_char == Some('u') {
                value.push('u');
                self.advance();
                if self.current_char == Some('e') {
                    value.push('e');
                    self.advance();
                    // Check if the next character is not a valid identifier character
                    if !self.current_char.is_some_and(Self::is_bare_key_char) {
                        return Ok(Token::Boolean(true));
                    }
                }
            }
        }

        // If it's not "true", read it as an identifier
        while let Some(c) = self.current_char {
            if !Self::is_bare_key_char(c) {
                break;
            }
            value.push(c);
            self.advance();
        }
        Ok(Token::String(value))
    }

    fn read_false_or_identifier(&mut self) -> Result<Token> {
        let mut value = String::new();
        // Safely get the current character
        if let Some(c) = self.current_char {
            value.push(c);
        } else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }
        self.advance();

        // Try to read "false"
        if self.current_char == Some('a') {
            value.push('a');
            self.advance();
            if self.current_char == Some('l') {
                value.push('l');
                self.advance();
                if self.current_char == Some('s') {
                    value.push('s');
                    self.advance();
                    if self.current_char == Some('e') {
                        value.push('e');
                        self.advance();
                        // Check if the next character is not a valid identifier character
                        if !self.current_char.is_some_and(Self::is_bare_key_char) {
                            return Ok(Token::Boolean(false));
                        }
                    }
                }
            }
        }

        // If it's not "false", read it as an identifier
        while let Some(c) = self.current_char {
            if !Self::is_bare_key_char(c) {
                break;
            }
            value.push(c);
            self.advance();
        }
        Ok(Token::String(value))
    }

    fn read_null_or_identifier(&mut self) -> Result<Token> {
        let mut value = String::new();

        // Read "null"
        if self.current_char == Some('n') {
            value.push('n');
            self.advance();
            if self.current_char == Some('u') {
                value.push('u');
                self.advance();
                if self.current_char == Some('l') {
                    value.push('l');
                    self.advance();
                    if self.current_char == Some('l') {
                        value.push('l');
                        self.advance();
                        // Check if the next character is not a valid identifier character
                        if !self.current_char.is_some_and(Self::is_bare_key_char) {
                            return Ok(Token::Null);
                        }
                    }
                }
            }
        }

        // If it's not "null", read it as an identifier
        while let Some(c) = self.current_char {
            if !Self::is_bare_key_char(c) {
                break;
            }
            value.push(c);
            self.advance();
        }
        Ok(Token::String(value))
    }
}
