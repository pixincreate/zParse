use std::fmt::Debug;
use zparse::error::{Error, ErrorKind, Result, Span};
use zparse::json::{Config, Event, Parser};
use zparse::value::{Object, Value};

fn fail<T>(message: String) -> Result<T> {
    Err(Error::with_message(
        ErrorKind::InvalidToken,
        Span::empty(),
        message,
    ))
}

fn ensure_eq<T: PartialEq + Debug>(left: T, right: T) -> Result<()> {
    if left == right {
        Ok(())
    } else {
        fail(format!("assertion failed: left={left:?} right={right:?}"))
    }
}

fn next_event_or_fail(parser: &mut Parser<'_>) -> Result<Option<Event>> {
    parser.next_event()
}

fn parse_value_or_fail(parser: &mut Parser<'_>) -> Result<Value> {
    parser.parse_value()
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.max_depth, 128);
    assert_eq!(config.max_size, 10 * 1024 * 1024);
    assert!(!config.allow_comments);
    assert!(!config.allow_trailing_commas);
}

#[test]
fn test_config_unlimited() {
    let config = Config::unlimited();
    assert_eq!(config.max_depth, 0);
    assert_eq!(config.max_size, 0);
    assert!(!config.allow_comments);
    assert!(!config.allow_trailing_commas);
}

#[test]
fn test_config_new() {
    let config = Config::new(64, 1024);
    assert_eq!(config.max_depth, 64);
    assert_eq!(config.max_size, 1024);
    assert!(!config.allow_comments);
    assert!(!config.allow_trailing_commas);
}

#[test]
fn test_comments_allowed() -> Result<()> {
    let input = b"// comment\n{\"a\": 1, /* inline */ \"b\": 2}\n";
    let config = Config::default().with_comments(true);
    let mut parser = Parser::with_config(input, config);
    let value = parser.parse_value()?;
    if let Value::Object(obj) = value {
        ensure_eq(obj.get("a"), Some(&Value::Number(1.0)))?;
        ensure_eq(obj.get("b"), Some(&Value::Number(2.0)))?;
    } else {
        return fail("expected object".to_string());
    }
    Ok(())
}

#[test]
fn test_trailing_comma_allowed() -> Result<()> {
    let input = b"{\"a\": 1,}\n";
    let config = Config::default().with_trailing_commas(true);
    let mut parser = Parser::with_config(input, config);
    let value = parser.parse_value()?;
    if let Value::Object(obj) = value {
        ensure_eq(obj.get("a"), Some(&Value::Number(1.0)))?;
    } else {
        return fail("expected object".to_string());
    }
    Ok(())
}

#[test]
fn test_trailing_comma_allowed_array() -> Result<()> {
    let input = b"[1, 2,]\n";
    let config = Config::default().with_trailing_commas(true);
    let mut parser = Parser::with_config(input, config);
    let value = parser.parse_value()?;
    if let Value::Array(arr) = value {
        ensure_eq(arr.len(), 2)?;
    } else {
        return fail("expected array".to_string());
    }
    Ok(())
}

#[test]
fn test_parser_new() {
    let input = b"null";
    let parser = Parser::new(input);
    assert_eq!(parser.config().max_depth, 128);
    assert_eq!(parser.depth(), 0);
    assert_eq!(parser.bytes_parsed(), 0);
}

#[test]
fn test_parser_with_config() {
    let input = b"null";
    let config = Config::new(32, 512);
    let parser = Parser::with_config(input, config);
    assert_eq!(parser.config().max_depth, 32);
    assert_eq!(parser.config().max_size, 512);
}

#[test]
fn test_parse_null() -> Result<()> {
    let input = b"null";
    let mut parser = Parser::new(input);

    let event = next_event_or_fail(&mut parser);
    let event = event?;
    ensure_eq(event, Some(Event::Value(Value::Null)))?;

    let event = next_event_or_fail(&mut parser);
    let event = event?;
    ensure_eq(event, None)?;
    Ok(())
}

#[test]
fn test_parse_bool() -> Result<()> {
    let input = b"true";
    let mut parser = Parser::new(input);

    let event = next_event_or_fail(&mut parser);
    let event = event?;
    ensure_eq(event, Some(Event::Value(Value::Bool(true))))?;
    Ok(())
}

#[test]
fn test_parse_number() -> Result<()> {
    let input = b"42.5";
    let mut parser = Parser::new(input);

    let event = next_event_or_fail(&mut parser);
    let event = event?;
    ensure_eq(event, Some(Event::Value(Value::Number(42.5))))?;
    Ok(())
}

#[test]
fn test_parse_string() -> Result<()> {
    let input = br#""hello world""#;
    let mut parser = Parser::new(input);

    let event = next_event_or_fail(&mut parser)?;
    ensure_eq(
        event,
        Some(Event::Value(Value::String("hello world".to_string()))),
    )?;
    Ok(())
}

#[test]
fn test_parse_empty_object() -> Result<()> {
    let input = b"{}";
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_empty_array() -> Result<()> {
    let input = b"[]";
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_simple_object() -> Result<()> {
    let input = br#"{"key": "value"}"#;
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("key".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::String("value".to_string()))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_simple_array() -> Result<()> {
    let input = b"[1, 2, 3]";
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(1.0))),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(2.0))),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(3.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_nested_object() -> Result<()> {
    let input = br#"{"outer": {"inner": 42}}"#;
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("outer".to_string())),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("inner".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(42.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_mixed() -> Result<()> {
    let input = br#"{"name": "test", "values": [1, 2], "flag": true}"#;
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("name".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::String("test".to_string()))),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("values".to_string())),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(1.0))),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(2.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("flag".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Bool(true))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_value_null() -> Result<()> {
    let input = b"null";
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;
    ensure_eq(value, Value::Null)?;
    Ok(())
}

#[test]
fn test_parse_value_bool() -> Result<()> {
    let input = b"true";
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;
    ensure_eq(value, Value::Bool(true))?;
    Ok(())
}

#[test]
fn test_parse_value_number() -> Result<()> {
    let input = b"123.456";
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;
    ensure_eq(value, Value::Number(123.456))?;
    Ok(())
}

#[test]
fn test_parse_value_string() -> Result<()> {
    let input = br#""test string""#;
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;
    ensure_eq(value, Value::String("test string".to_string()))?;
    Ok(())
}

#[test]
fn test_parse_value_array() -> Result<()> {
    let input = b"[1, 2, 3]";
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;
    let expected = Value::Array(vec![1.0.into(), 2.0.into(), 3.0.into()].into());
    ensure_eq(value, expected)?;
    Ok(())
}

#[test]
fn test_parse_value_object() -> Result<()> {
    let input = br#"{"a": 1, "b": 2}"#;
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;

    let mut expected = Object::new();
    expected.insert("a", 1i32);
    expected.insert("b", 2i32);
    ensure_eq(value, Value::Object(expected))?;
    Ok(())
}

#[test]
fn test_parse_value_nested() -> Result<()> {
    let input = br#"{"arr": [1, {"nested": "value"}]}"#;
    let mut parser = Parser::new(input);
    let value = parse_value_or_fail(&mut parser)?;

    if let Value::Object(obj) = value {
        if !obj.contains_key("arr") {
            return fail("Expected key 'arr'".to_string());
        }
        if let Some(Value::Array(arr)) = obj.get("arr") {
            ensure_eq(arr.len(), 2)?;
            ensure_eq(arr.get(0), Some(&Value::Number(1.0)))?;
        } else {
            return fail("Expected array".to_string());
        }
    } else {
        return fail("Expected object".to_string());
    }
    Ok(())
}

#[test]
fn test_depth_limit() -> Result<()> {
    let input = br#"{"a": {"b": {"c": 1}}}"#;
    let config = Config::new(2, 0); // max depth of 2
    let mut parser = Parser::with_config(input, config);

    // Should fail when trying to enter third level
    let result = parser.parse_value();
    if !matches!(
        result,
        Err(err) if matches!(err.kind(), ErrorKind::MaxDepthExceeded { max: 2 })
    ) {
        return fail("Expected max depth error".to_string());
    }
    Ok(())
}

#[test]
fn test_size_limit() -> Result<()> {
    let input = b"1234567890";
    let config = Config::new(0, 5); // max size of 5 bytes
    let mut parser = Parser::with_config(input, config);

    let result = parser.parse_value();
    if !matches!(
        result,
        Err(err) if matches!(err.kind(), ErrorKind::MaxSizeExceeded { max: 5 })
    ) {
        return fail("Expected max size error".to_string());
    }
    Ok(())
}

#[test]
fn test_parse_object_with_multiple_keys() -> Result<()> {
    let input = br#"{"a": 1, "b": 2, "c": 3}"#;
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("a".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(1.0))),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("b".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(2.0))),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("c".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(3.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_array_with_nested_objects() -> Result<()> {
    let input = br#"[{"x": 1}, {"y": 2}]"#;
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("x".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(1.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Key("y".to_string())),
    )?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(2.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ObjectEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}

#[test]
fn test_parse_deeply_nested() -> Result<()> {
    let input = br#"[[[[1]]]]"#;
    let mut parser = Parser::new(input);

    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayStart))?;
    ensure_eq(
        next_event_or_fail(&mut parser)?,
        Some(Event::Value(Value::Number(1.0))),
    )?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, Some(Event::ArrayEnd))?;
    ensure_eq(next_event_or_fail(&mut parser)?, None)?;
    Ok(())
}
