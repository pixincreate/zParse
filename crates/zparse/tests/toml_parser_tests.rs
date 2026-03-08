use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use zparse::error::{Error, ErrorKind, Result};
use zparse::toml::parser::{Config, Parser};
use zparse::{Span, TomlDatetime, Value};

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
fn test_parse_basic_table() -> Result<()> {
    let input = b"title = \"TOML\"\n[owner]\nname = \"Tom\"\n";
    let mut parser = Parser::new(input);
    let value = parser.parse()?;

    if let Value::Object(obj) = value {
        ensure_eq(obj.get("title"), Some(&Value::String("TOML".to_string())))?;
        let owner = obj.get("owner");
        match owner {
            Some(Value::Object(owner)) => {
                ensure_eq(owner.get("name"), Some(&Value::String("Tom".to_string())))?;
            }
            _ => {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "missing owner table".to_string(),
                ));
            }
        }
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
fn test_parse_datetime_values() -> Result<()> {
    let input = b"offset = 1979-05-27T07:32:00Z\nlocal_dt = 1979-05-27T07:32:00\nlocal_date = 1979-05-27\nlocal_time = 07:32:00\n";
    let mut parser = Parser::new(input);
    let value = parser.parse()?;

    if let Value::Object(obj) = value {
        let offset = obj.get("offset");
        let expected_offset =
            OffsetDateTime::parse("1979-05-27T07:32:00Z", &Rfc3339).map_err(|_| {
                Error::with_message(
                    ErrorKind::InvalidDatetime,
                    Span::empty(),
                    "failed to parse offset datetime".to_string(),
                )
            })?;
        ensure_eq(
            offset,
            Some(&Value::Datetime(TomlDatetime::OffsetDateTime(
                expected_offset,
            ))),
        )?;

        let local_dt = PrimitiveDateTime::parse(
            "1979-05-27T07:32:00",
            &format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        )
        .map_err(|_| {
            Error::with_message(
                ErrorKind::InvalidDatetime,
                Span::empty(),
                "failed to parse local datetime".to_string(),
            )
        })?;
        ensure_eq(
            obj.get("local_dt"),
            Some(&Value::Datetime(TomlDatetime::LocalDateTime(local_dt))),
        )?;

        let local_date = Date::parse("1979-05-27", &format_description!("[year]-[month]-[day]"))
            .map_err(|_| {
                Error::with_message(
                    ErrorKind::InvalidDatetime,
                    Span::empty(),
                    "failed to parse local date".to_string(),
                )
            })?;
        ensure_eq(
            obj.get("local_date"),
            Some(&Value::Datetime(TomlDatetime::LocalDate(local_date))),
        )?;

        let local_time = Time::parse("07:32:00", &format_description!("[hour]:[minute]:[second]"))
            .map_err(|_| {
                Error::with_message(
                    ErrorKind::InvalidDatetime,
                    Span::empty(),
                    "failed to parse local time".to_string(),
                )
            })?;
        ensure_eq(
            obj.get("local_time"),
            Some(&Value::Datetime(TomlDatetime::LocalTime(local_time))),
        )?;
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
fn test_parse_inline_table() -> Result<()> {
    let input = b"point = { x = 1, y = 2 }\n";
    let mut parser = Parser::new(input);
    let value = parser.parse()?;

    if let Value::Object(obj) = value {
        match obj.get("point") {
            Some(Value::Object(point)) => {
                ensure_eq(point.get("x"), Some(&Value::Number(1.0)))?;
                ensure_eq(point.get("y"), Some(&Value::Number(2.0)))?;
            }
            _ => {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "missing point table".to_string(),
                ));
            }
        }
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
fn test_parse_complex_toml_document() -> Result<()> {
    let input = b"title = \"Complex\"\n\
[owner]\n\
name = \"Tom\"\n\
dob = 1979-05-27T07:32:00Z\n\
\n\
[database]\n\
enabled = true\n\
ports = [ 8000, 8001, 8002 ]\n\
temp_targets = { cpu = 79.5, case = 72.0 }\n\
\n\
[[products]]\n\
name = \"Hammer\"\n\
sku = 738594937\n\
\n\
[[products]]\n\
name = \"Nail\"\n\
sku = 284758393\n\
color = \"gray\"\n";

    let mut parser = Parser::new(input);
    let value = parser.parse()?;

    if let Value::Object(obj) = value {
        ensure_eq(
            obj.get("title"),
            Some(&Value::String("Complex".to_string())),
        )?;

        match obj.get("database") {
            Some(Value::Object(database)) => {
                ensure_eq(database.get("enabled"), Some(&Value::Bool(true)))?;
                match database.get("ports") {
                    Some(Value::Array(ports)) => ensure_eq(ports.len(), 3)?,
                    _ => {
                        return Err(Error::with_message(
                            ErrorKind::InvalidToken,
                            Span::empty(),
                            "expected ports array".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "expected database table".to_string(),
                ));
            }
        }

        match obj.get("products") {
            Some(Value::Array(products)) => ensure_eq(products.len(), 2)?,
            _ => {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "expected products array".to_string(),
                ));
            }
        }
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
fn test_size_limit_counts_ignorable_prefix() -> Result<()> {
    let input = b"# long comment prefix\nkey = 1\n";
    let config = Config::new(0, 10);
    let mut parser = Parser::with_config(input, config);

    let result = parser.parse();
    if !matches!(
        result,
        Err(err) if matches!(err.kind(), ErrorKind::MaxSizeExceeded { max: 10 })
    ) {
        return Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "expected max-size error counting ignorable prefix".to_string(),
        ));
    }

    Ok(())
}
