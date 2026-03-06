use zparse::error::{Error, ErrorKind, Result};
use zparse::xml::parser::Parser;
use zparse::{Span, XmlContent};

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
