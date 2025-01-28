//! Lexical analyzer for parsing JSON and TOML documents.
//!
//! This module is split into smaller files to handle specific tasks like
//! parsing strings (string_parser.rs) and parsing numbers (number_parser.rs).

pub mod number_parser;
pub mod string_parser;

use number_parser::read_number;
use string_parser::read_string;

use crate::enums::Token;
use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::config::ParserConfig;

/// The core Lexer struct that other modules will use
#[derive(Debug)]
pub struct Lexer {
    /// Input text as a character array
    pub(crate) input: Vec<char>,
    /// Current position in the input
    pub(crate) position: usize,
    /// Current character being processed
    pub(crate) current_char: Option<char>,
    /// Whether we're in JSON mode (affects string quoting rules, bare keys, etc.)
    pub(crate) is_json_mode: bool,
    /// Configuration for the parser
    pub(crate) config: ParserConfig,
}

impl Lexer {
    /// Creates a new TOML lexer from input text
    pub fn new(input: &str) -> Self {
        let input_vec: Vec<char> = input.chars().collect();
        let current_char = input_vec.first().copied();
        Self {
            input: input_vec,
            position: 0,
            current_char,
            is_json_mode: false,
            config: ParserConfig::default(),
        }
    }

    /// Creates a new JSON lexer from input text
    pub fn new_json(input: &str) -> Self {
        let mut lexer = Self::new(input);
        lexer.is_json_mode = true;
        lexer
    }

    /// Moves to the next character in the input
    pub(crate) fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// Skips whitespace characters in the input
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    /// Produces the next token from the input
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
                    // Route to dedicated string parsing
                    let s = read_string(self)?;
                    Ok(Token::String(s))
                }
                '0'..='9' | '-' | '_' => {
                    // Route to dedicated number parsing
                    let n = read_number(self)?;
                    Ok(Token::Number(n))
                }
                't' => self.read_true_or_identifier(),
                'f' => self.read_false_or_identifier(),
                'n' => self.read_null_or_identifier(),
                _ if Self::is_bare_key_start(c) => {
                    // In JSON mode, we forbid bare keys
                    if self.is_json_mode {
                        Err(ParseError::new(ParseErrorKind::InvalidToken(format!(
                            "Unexpected char '{}'. JSON requires quoted strings",
                            c
                        ))))
                    } else {
                        // Parse a bare key
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
            Err(ParseError::new(ParseErrorKind::InvalidKey(
                "Empty key".to_string(),
            )))
        } else {
            Ok(key)
        }
    }

    fn read_true_or_identifier(&mut self) -> Result<Token> {
        let mut value = String::new();
        if let Some(c) = self.current_char {
            value.push(c);
            self.advance();
        } else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }
        // Attempt “true”
        if self.current_char == Some('r') {
            value.push('r');
            self.advance();
            if self.current_char == Some('u') {
                value.push('u');
                self.advance();
                if self.current_char == Some('e') {
                    value.push('e');
                    self.advance();
                    // Must not continue to a valid identifier char
                    if !self.current_char.is_some_and(Self::is_bare_key_char) {
                        return Ok(Token::Boolean(true));
                    }
                }
            }
        }
        // Otherwise, read remainder as identifier
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
        if let Some(c) = self.current_char {
            value.push(c);
            self.advance();
        } else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }
        // Attempt “false”
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
                        if !self.current_char.is_some_and(Self::is_bare_key_char) {
                            return Ok(Token::Boolean(false));
                        }
                    }
                }
            }
        }
        // Otherwise, identifier
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
        // We already have 'n'
        if let Some(c) = self.current_char {
            value.push(c);
            self.advance();
        } else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
        }
        // Attempt “null”
        if self.current_char == Some('u') {
            value.push('u');
            self.advance();
            if self.current_char == Some('l') {
                value.push('l');
                self.advance();
                if self.current_char == Some('l') {
                    value.push('l');
                    self.advance();
                    if !self.current_char.is_some_and(Self::is_bare_key_char) {
                        return Ok(Token::Null);
                    }
                }
            }
        }
        // Otherwise, read remainder as identifier
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
