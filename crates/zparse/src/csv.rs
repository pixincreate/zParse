pub mod parser;

pub use parser::Parser;

use crate::value::Value;

pub fn from_csv_str(s: &str) -> crate::error::Result<Value> {
    let mut parser = Parser::new(s.as_bytes());
    parser.parse()
}

pub fn from_csv_bytes(bytes: &[u8]) -> crate::error::Result<Value> {
    let mut parser = Parser::new(bytes);
    parser.parse()
}

pub fn from_csv_str_with_delimiter(s: &str, delimiter: u8) -> crate::error::Result<Value> {
    let mut parser = Parser::with_delimiter(s.as_bytes(), delimiter);
    parser.parse()
}

pub fn from_csv_bytes_with_delimiter(bytes: &[u8], delimiter: u8) -> crate::error::Result<Value> {
    let mut parser = Parser::with_delimiter(bytes, delimiter);
    parser.parse()
}

pub(crate) fn infer_primitive_value(input: &str) -> Option<Value> {
    if input.is_empty() || input.eq_ignore_ascii_case("null") {
        return Some(Value::Null);
    }

    if input.eq_ignore_ascii_case("true") {
        return Some(Value::Bool(true));
    }

    if input.eq_ignore_ascii_case("false") {
        return Some(Value::Bool(false));
    }

    if input.parse::<i64>().is_ok()
        && let Ok(number) = input.parse::<f64>()
        && number.is_finite()
    {
        return Some(Value::Number(number));
    }

    if let Ok(float) = input.parse::<f64>()
        && float.is_finite()
    {
        return Some(Value::Number(float));
    }

    None
}
