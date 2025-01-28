use super::Lexer;
use crate::error::{ParseError, ParseErrorKind, Result};

pub(crate) fn read_string(lexer: &mut Lexer) -> Result<String> {
    // Skip the opening quote
    lexer.advance();

    let mut result = String::new();
    let max_length = lexer.config.max_string_length;

    while let Some(c) = lexer.current_char {
        if result.len() >= max_length {
            return Err(ParseError::new(ParseErrorKind::MaxStringLengthExceeded));
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
                        _ => {
                            return Err(ParseError::new(ParseErrorKind::InvalidEscape(
                                escape_char,
                            )));
                        }
                    };
                    result.push(escaped);
                    lexer.advance();
                } else {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedEOF));
                }
            }
            _ => {
                result.push(c);
                lexer.advance();
            }
        }
    }

    // If we exit the while loop, we ran out of characters before finding a closing quote
    Err(ParseError::new(ParseErrorKind::UnexpectedEOF))
}
