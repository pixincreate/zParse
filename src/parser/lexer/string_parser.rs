use super::Lexer;
use crate::error::{LexicalError, ParseError, ParseErrorKind, Result};

pub(crate) fn read_string(lexer: &mut Lexer) -> Result<String> {
    // Skip the opening quote
    lexer.advance();

    let mut result = String::new();
    let max_length = lexer.config.max_string_length;

    while let Some(c) = lexer.current_char {
        if result.len() >= max_length {
            return Err(ParseError::new(ParseErrorKind::Security(
                crate::error::SecurityError::MaxStringLengthExceeded,
            )));
        }

        match c {
            '"' => {
                // Reached the closing quote
                lexer.advance();
                return Ok(result);
            }
            '\\' => {
                // Handle escaped sequences
                lexer.advance(); // consume '\'
                if let Some(escape_char) = lexer.current_char {
                    let escaped = match escape_char {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        'u' => return parse_unicode_escape(lexer),
                        _ => {
                            return Err(ParseError::new(ParseErrorKind::Lexical(
                                LexicalError::InvalidEscape(escape_char),
                            )));
                        }
                    };
                    result.push(escaped);
                    lexer.advance();
                } else {
                    return Err(ParseError::new(ParseErrorKind::Lexical(
                        LexicalError::UnexpectedEOF,
                    )));
                }
            }
            _ => {
                result.push(c);
                lexer.advance();
            }
        }
    }

    // If we exit the while loop, we ran out of characters before finding a closing quote
    Err(ParseError::new(ParseErrorKind::Lexical(
        LexicalError::UnexpectedEOF,
    )))
}

fn parse_unicode_escape(lexer: &mut Lexer) -> Result<String> {
    lexer.advance(); // skip 'u'

    // Unicode escapes must be exactly 4 hexadecimal digits
    let mut code_point = 0u32;
    for _ in 0..4 {
        match lexer.current_char {
            Some(c) => {
                let digit = match c {
                    '0'..='9' => c.to_digit(16).unwrap_or(0),
                    'a'..='f' => c.to_digit(16).unwrap_or(0),
                    'A'..='F' => c.to_digit(16).unwrap_or(0),
                    _ => {
                        return Err(ParseError::new(ParseErrorKind::Lexical(
                            LexicalError::InvalidUnicode,
                        )));
                    }
                };
                code_point = code_point.saturating_mul(16).saturating_add(digit);
                lexer.advance();
            }
            None => {
                return Err(ParseError::new(ParseErrorKind::Lexical(
                    LexicalError::UnexpectedEOF,
                )))
            }
        }
    }

    // Convert the code point to a character
    match char::from_u32(code_point) {
        Some(c) => Ok(c.to_string()),
        None => Err(ParseError::new(ParseErrorKind::Lexical(
            LexicalError::InvalidUnicode,
        ))),
    }
}
