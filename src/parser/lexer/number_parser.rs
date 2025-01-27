use super::Lexer;
use crate::error::{ParseError, ParseErrorKind, Result};

pub(crate) fn read_number(lexer: &mut Lexer) -> Result<f64> {
    let mut number_str = String::new();
    let mut is_float = false;
    let mut has_digits = false;
    let mut previous_char_was_underscore = false;

    // Handle optional leading '-'
    if lexer.current_char == Some('-') {
        number_str.push('-');
        lexer.advance();
    }

    // Parse main part of the number
    while let Some(c) = lexer.current_char {
        match c {
            '0'..='9' => {
                number_str.push(c);
                has_digits = true;
                previous_char_was_underscore = false;
                lexer.advance();
            }
            '_' => {
                // Underscores must follow a digit (and not another underscore)
                if !has_digits || previous_char_was_underscore {
                    return Err(ParseError::new(ParseErrorKind::InvalidNumber(number_str)));
                }
                previous_char_was_underscore = true;
                lexer.advance();
            }
            '.' => {
                // Only allow a single decimal point, and not immediately after '_'
                if is_float || previous_char_was_underscore {
                    return Err(ParseError::new(ParseErrorKind::InvalidNumber(number_str)));
                }
                is_float = true;
                previous_char_was_underscore = false;
                number_str.push(c);
                lexer.advance();
            }
            _ => break,
        }
    }

    if !has_digits {
        return Err(ParseError::new(ParseErrorKind::InvalidNumber(number_str)));
    }

    // Remove underscores from the string that will be parsed
    let clean_number_str: String = number_str.chars().filter(|&c| c != '_').collect();

    // Attempt to parse
    clean_number_str
        .parse::<f64>()
        .map_err(|_| ParseError::new(ParseErrorKind::InvalidNumber(number_str)))
}
