use zparse::error::{Error, ErrorKind, Result};
use zparse::yaml::parser::Parser;
use zparse::{Span, Value};

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
