//! XML parser implementation

use indexmap::IndexMap;

use crate::error::{Error, ErrorKind, Pos, Result, Span};
use crate::lexer::Cursor;
use crate::xml::model::{Content, Document, Element};

/// XML parser
#[derive(Debug)]
pub struct Parser<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new XML parser
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }

    /// Parse an XML document
    pub fn parse(&mut self) -> Result<Document> {
        self.skip_whitespace();
        let root = self.parse_element()?;
        self.skip_whitespace();

        if !self.cursor.is_eof() {
            return Err(Error::at(
                ErrorKind::InvalidToken,
                self.cursor.position().offset,
                self.cursor.position().line,
                self.cursor.position().col,
            ));
        }

        Ok(Document { root })
    }

    fn parse_element(&mut self) -> Result<Element> {
        self.expect_byte(b'<')?;

        if self.cursor.current() == Some(b'?') {
            self.skip_processing_instruction()?;
            return self.parse_element();
        }

        if self.cursor.current() == Some(b'!') {
            self.skip_declaration_or_comment()?;
            return self.parse_element();
        }

        if self.cursor.current() == Some(b'/') {
            return Err(self.error_here("unexpected closing tag"));
        }

        let name = self.parse_name()?;
        let attributes = self.parse_attributes()?;

        if self.cursor.current() == Some(b'/') {
            self.cursor.advance();
            self.expect_byte(b'>')?;
            return Ok(Element {
                name,
                attributes,
                children: Vec::new(),
            });
        }

        self.expect_byte(b'>')?;

        let mut children = Vec::new();
        loop {
            if self.cursor.current() == Some(b'<') && self.cursor.peek(1) == Some(b'/') {
                self.cursor.advance_by(2);
                let close_name = self.parse_name()?;
                if close_name != name {
                    return Err(self.error_here("mismatched closing tag"));
                }
                self.skip_whitespace();
                self.expect_byte(b'>')?;
                break;
            }

            if self.cursor.current() == Some(b'<') {
                let child = self.parse_element()?;
                children.push(Content::Element(child));
                continue;
            }

            if self.cursor.is_eof() {
                return Err(self.error_here("unterminated element"));
            }

            if let Some(text) = self.parse_text()? {
                children.push(Content::Text(text));
            }
        }

        Ok(Element {
            name,
            attributes,
            children,
        })
    }

    fn parse_attributes(&mut self) -> Result<IndexMap<String, String>> {
        let mut attrs = IndexMap::new();

        loop {
            self.skip_whitespace();
            match self.cursor.current() {
                Some(b'/') | Some(b'>') => break,
                Some(_) => {}
                None => return Err(self.error_here("unexpected end of input")),
            }

            let name = self.parse_name()?;
            self.skip_whitespace();
            self.expect_byte(b'=')?;
            self.skip_whitespace();
            let value = self.parse_attribute_value()?;

            if attrs.contains_key(&name) {
                return Err(self.error_here("duplicate attribute"));
            }
            attrs.insert(name, value);
        }

        Ok(attrs)
    }

    fn parse_attribute_value(&mut self) -> Result<String> {
        let quote = match self.cursor.current() {
            Some(b'"') => b'"',
            Some(b'\'') => b'\'',
            _ => return Err(self.error_here("expected quoted attribute value")),
        };
        self.cursor.advance();

        let start = self.cursor.pos();
        while let Some(b) = self.cursor.current() {
            if b == quote {
                let raw = self.cursor.slice_from(start);
                self.cursor.advance();
                let text = bytes_to_string(raw)?;
                return decode_entities(&text);
            }
            self.cursor.advance();
        }

        Err(self.error_here("unterminated attribute value"))
    }

    fn parse_text(&mut self) -> Result<Option<String>> {
        let start = self.cursor.pos();
        while let Some(b) = self.cursor.current() {
            if b == b'<' {
                break;
            }
            self.cursor.advance();
        }

        let raw = self.cursor.slice_from(start);
        let text = bytes_to_string(raw)?;
        let text = decode_entities(&text)?;

        if text.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(text))
        }
    }

    fn parse_name(&mut self) -> Result<String> {
        let start_pos = self.cursor.position();
        let start = self.cursor.pos();

        let Some(first) = self.cursor.current() else {
            return Err(self.error_here("expected name"));
        };
        if !is_name_start(first) {
            return Err(Error::at(
                ErrorKind::InvalidToken,
                start_pos.offset,
                start_pos.line,
                start_pos.col,
            ));
        }

        self.cursor.advance();
        while let Some(b) = self.cursor.current() {
            if is_name_char(b) {
                self.cursor.advance();
            } else {
                break;
            }
        }

        let raw = self.cursor.slice_from(start);
        bytes_to_string(raw)
    }

    fn skip_declaration_or_comment(&mut self) -> Result<()> {
        // cursor currently at '!'
        if self.cursor.peek(1) == Some(b'-') && self.cursor.peek(2) == Some(b'-') {
            self.cursor.advance_by(3);
            self.skip_until(b"-->")?;
            return Ok(());
        }

        if self.cursor.peek(1) == Some(b'[')
            && self.cursor.peek(2) == Some(b'C')
            && self.cursor.peek(3) == Some(b'D')
        {
            self.cursor.advance_by(2);
            self.skip_until(b"]]>")?;
            return Ok(());
        }

        self.skip_until(b">")
    }

    fn skip_processing_instruction(&mut self) -> Result<()> {
        // cursor currently at '?'
        self.cursor.advance();
        self.skip_until(b"?>")
    }

    fn skip_until(&mut self, pattern: &[u8]) -> Result<()> {
        while self.cursor.current().is_some() {
            if self.cursor.peek_bytes(pattern.len()) == Some(pattern) {
                self.cursor.advance_by(pattern.len());
                return Ok(());
            }
            self.cursor.advance();
        }
        Err(self.error_here("unterminated markup"))
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        if self.cursor.current() == Some(expected) {
            self.cursor.advance();
            Ok(())
        } else {
            Err(self.error_here("unexpected token"))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.cursor.current() {
            if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
                self.cursor.advance();
            } else {
                break;
            }
        }
    }

    fn error_here(&self, message: &str) -> Error {
        let pos = self.cursor.position();
        Error::with_message(
            ErrorKind::InvalidToken,
            Span::new(Pos::new(pos.offset, pos.line, pos.col), pos),
            message.to_string(),
        )
    }
}

fn bytes_to_string(bytes: &[u8]) -> Result<String> {
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "invalid utf-8".to_string(),
            )
        })
}

fn is_name_start(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'_' | b':')
}

fn is_name_char(b: u8) -> bool {
    is_name_start(b) || matches!(b, b'0'..=b'9' | b'-' | b'.')
}

fn decode_entities(input: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '&' {
            result.push(ch);
            continue;
        }

        let mut entity = String::new();
        for next in chars.by_ref() {
            if next == ';' {
                break;
            }
            entity.push(next);
        }

        let decoded = match entity.as_str() {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => decode_numeric_entity(&entity),
        };

        match decoded {
            Some(ch) => result.push(ch),
            None => {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "invalid xml entity".to_string(),
                ));
            }
        }
    }

    Ok(result)
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
    if let Some(hex) = entity.strip_prefix("#x") {
        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
    } else if let Some(dec) = entity.strip_prefix('#') {
        dec.parse::<u32>().ok().and_then(char::from_u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        ensure_eq(doc.root.name, "root".to_string())?;
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
            Some(Content::Element(child)) => {
                ensure_eq(child.name.clone(), "child".to_string())?;
                match child.children.first() {
                    Some(Content::Text(text)) => {
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
            Some(Content::Element(child)) => {
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
}
