use zparse::error::{Error, ErrorKind, Result};
use zparse::xml::parser::Parser;
use zparse::{Span, XmlConfig, XmlContent};

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
fn test_parse_simple_element() -> Result<()> {
    let input = b"<root></root>";
    let mut parser = Parser::new(input);
    let doc = parser.parse()?;

    ensure_eq(doc.root.name.as_str(), "root")?;
    ensure_eq(doc.root.children.len(), 0)?;
    Ok(())
}

#[test]
fn test_parse_with_attributes() -> Result<()> {
    let input = b"<root id=\"1\" name='test'></root>";
    let mut parser = Parser::new(input);
    let doc = parser.parse()?;

    ensure_eq(doc.root.attributes.get("id"), Some(&"1".to_string()))?;
    ensure_eq(doc.root.attributes.get("name"), Some(&"test".to_string()))?;
    Ok(())
}

#[test]
fn test_parse_nested() -> Result<()> {
    let input = b"<root><child>text</child></root>";
    let mut parser = Parser::new(input);
    let doc = parser.parse()?;

    match doc.root.children.first() {
        Some(XmlContent::Element(child)) => {
            ensure_eq(child.name.clone(), "child".to_string())?;
            match child.children.first() {
                Some(XmlContent::Text(text)) => {
                    ensure_eq(text, &"text".to_string())?;
                }
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidToken,
                        Span::empty(),
                        "expected text".to_string(),
                    ));
                }
            }
        }
        _ => {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "expected child element".to_string(),
            ));
        }
    }

    Ok(())
}

#[test]
fn test_parse_self_closing() -> Result<()> {
    let input = b"<root><child /></root>";
    let mut parser = Parser::new(input);
    let doc = parser.parse()?;

    match doc.root.children.first() {
        Some(XmlContent::Element(child)) => {
            ensure_eq(child.name.clone(), "child".to_string())?;
            ensure_eq(child.children.len(), 0)?;
        }
        _ => {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "expected child element".to_string(),
            ));
        }
    }

    Ok(())
}

#[test]
fn test_parse_complex_xml_document() -> Result<()> {
    let input = b"<catalog region=\"global\"><book id=\"b1\"><title>Rust</title><authors><author>Ann</author><author>Bob</author></authors><price currency=\"USD\">39.99</price></book><book id=\"b2\"><title>XML</title><price currency=\"EUR\">29.50</price></book></catalog>";
    let mut parser = Parser::new(input);
    let doc = parser.parse()?;

    ensure_eq(doc.root.name.as_str(), "catalog")?;
    ensure_eq(
        doc.root.attributes.get("region"),
        Some(&"global".to_string()),
    )?;
    ensure_eq(doc.root.children.len(), 2)?;

    match doc.root.children.first() {
        Some(XmlContent::Element(book)) => {
            ensure_eq(book.name.as_str(), "book")?;
            ensure_eq(book.attributes.get("id"), Some(&"b1".to_string()))?;
            ensure_eq(book.children.len(), 3)?;
        }
        _ => {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "expected first book element".to_string(),
            ));
        }
    }

    Ok(())
}

#[test]
<<<<<<< HEAD
fn test_parse_unterminated_element_returns_error() -> Result<()> {
    let input = b"<root><child>unclosed";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    if result.is_err() {
        Ok(())
    } else {
        Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "expected error for unterminated element".to_string(),
        ))
    }
}

#[test]
fn test_parse_mismatched_tags_returns_error() -> Result<()> {
    let input = b"<root><child></wrong></root>";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    if result.is_err() {
        Ok(())
    } else {
        Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "expected error for mismatched tags".to_string(),
        ))
    }
}

#[test]
fn test_convert_malformed_xml_to_json_returns_error() -> Result<()> {
    use zparse::convert::{Format, convert};
    let input = "<root><row><name>test";
    let result = convert(input, Format::Xml, Format::Json);
    if result.is_err() {
        Ok(())
    } else {
        Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "malformed XML should return error, got Ok".to_string(),
        ))
    }
}
}

#[test]
fn test_from_xml_str_with_config_wires_through() -> Result<()> {
    let config = XmlConfig::default();
    let doc = zparse::from_xml_str_with_config("<root><child /></root>", config)?;
    ensure_eq(doc.root.name.as_str(), "root")?;
    ensure_eq(doc.root.children.len(), 1)?;
    Ok(())
}

#[test]
fn test_xml_parser_max_size_guard_with_config() -> Result<()> {
    let config = XmlConfig::new(4);
    let err = match zparse::from_xml_str_with_config("<root></root>", config) {
        Ok(doc) => {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                format!("expected max-size guard error, got {doc:?}"),
            ));
        }
        Err(err) => err,
    };

    match err.kind() {
        ErrorKind::MaxSizeExceeded { max } => ensure_eq(*max, 4usize)?,
        other => {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                format!("expected MaxSizeExceeded, got {other:?}"),
            ));
        }
    }

    Ok(())
}
