//! YAML lexer with indentation-aware tokens

use std::collections::VecDeque;

use crate::error::{Error, ErrorKind, Result, Span};

/// YAML token kinds
#[derive(Clone, Debug, PartialEq)]
pub enum YamlTokenKind {
    Indent,
    Dedent,
    Dash,
    Colon,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Scalar(String),
    Newline,
    Eof,
}

/// YAML token with span
#[derive(Clone, Debug, PartialEq)]
pub struct YamlToken {
    pub kind: YamlTokenKind,
    pub span: Span,
}

impl YamlToken {
    pub const fn new(kind: YamlTokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// YAML lexer
#[derive(Clone, Debug)]
pub struct YamlLexer<'a> {
    input: &'a [u8],
    index: usize,
    line: u32,
    indent_stack: Vec<usize>,
    pending: VecDeque<YamlToken>,
}

impl<'a> YamlLexer<'a> {
    /// Create a new YAML lexer
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            index: 0,
            line: 1,
            indent_stack: vec![0],
            pending: VecDeque::new(),
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<YamlToken> {
        if let Some(token) = self.pending.pop_front() {
            return Ok(token);
        }

        if self.index >= self.input.len() {
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                return Ok(YamlToken::new(YamlTokenKind::Dedent, Span::empty()));
            }
            return Ok(YamlToken::new(YamlTokenKind::Eof, Span::empty()));
        }

        let line_start = self.index;
        let mut line_end = self.index;
        while let Some(byte) = self.input.get(line_end) {
            if *byte == b'\n' {
                break;
            }
            line_end = line_end.saturating_add(1);
        }

        let line_bytes = self.input.get(line_start..line_end).ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "invalid line range".to_string(),
            )
        })?;
        self.index = if line_end < self.input.len() {
            line_end.saturating_add(1)
        } else {
            line_end
        };

        let line_str = std::str::from_utf8(line_bytes).map_err(|_| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "invalid utf-8 in yaml".to_string(),
            )
        })?;

        let (indent, content) = split_indent(line_str)?;
        let content = strip_comment(content);
        if content.trim().is_empty() {
            self.line = self.line.saturating_add(1);
            return self.next_token();
        }

        let current_indent = *self.indent_stack.last().unwrap_or(&0);
        if indent > current_indent {
            self.indent_stack.push(indent);
            self.pending
                .push_back(YamlToken::new(YamlTokenKind::Indent, Span::empty()));
        } else if indent < current_indent {
            while let Some(last) = self.indent_stack.last() {
                if *last == indent {
                    break;
                }
                self.indent_stack.pop();
                self.pending
                    .push_back(YamlToken::new(YamlTokenKind::Dedent, Span::empty()));
            }
            if *self.indent_stack.last().unwrap_or(&0) != indent {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "invalid indentation".to_string(),
                ));
            }
        }

        let tokens = lex_line(content)?;
        for token in tokens {
            self.pending.push_back(token);
        }
        self.pending
            .push_back(YamlToken::new(YamlTokenKind::Newline, Span::empty()));

        self.line = self.line.saturating_add(1);
        self.pending.pop_front().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "lexer error".to_string(),
            )
        })
    }
}

fn split_indent(line: &str) -> Result<(usize, &str)> {
    let mut indent = 0_usize;
    for ch in line.chars() {
        match ch {
            ' ' => indent = indent.saturating_add(1),
            '\t' => {
                return Err(Error::with_message(
                    ErrorKind::InvalidToken,
                    Span::empty(),
                    "tabs not allowed for indentation".to_string(),
                ));
            }
            _ => break,
        }
    }
    Ok((indent, &line[indent..]))
}

fn strip_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_escape = false;
    for (idx, ch) in line.char_indices() {
        match ch {
            '\\' if in_double => {
                prev_escape = !prev_escape;
                continue;
            }
            '"' if !in_single && !prev_escape => {
                in_double = !in_double;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '#' if !in_single && !in_double => return &line[..idx],
            _ => {}
        }
        prev_escape = false;
    }
    line
}

fn lex_line(line: &str) -> Result<Vec<YamlToken>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if let Some(rest) = trimmed.strip_prefix('-') {
        let rest = rest.strip_prefix(' ').unwrap_or(rest).trim();
        let mut tokens = Vec::new();
        tokens.push(YamlToken::new(YamlTokenKind::Dash, Span::empty()));
        if !rest.is_empty() {
            tokens.extend(lex_value_tokens(rest)?);
        }
        return Ok(tokens);
    }

    lex_mapping_or_scalar(trimmed)
}

fn lex_mapping_or_scalar(line: &str) -> Result<Vec<YamlToken>> {
    if let Some((key, value)) = split_key_value(line)? {
        let mut tokens = Vec::new();
        let key = parse_scalar(key)?;
        tokens.push(YamlToken::new(YamlTokenKind::Scalar(key), Span::empty()));
        tokens.push(YamlToken::new(YamlTokenKind::Colon, Span::empty()));
        if let Some(value) = value {
            tokens.extend(lex_value_tokens(value)?);
        }
        return Ok(tokens);
    }

    lex_value_tokens(line)
}

fn lex_value_tokens(line: &str) -> Result<Vec<YamlToken>> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        match ch {
            '[' => {
                chars.next();
                tokens.push(YamlToken::new(YamlTokenKind::LeftBracket, Span::empty()));
            }
            ']' => {
                chars.next();
                tokens.push(YamlToken::new(YamlTokenKind::RightBracket, Span::empty()));
            }
            '{' => {
                chars.next();
                tokens.push(YamlToken::new(YamlTokenKind::LeftBrace, Span::empty()));
            }
            '}' => {
                chars.next();
                tokens.push(YamlToken::new(YamlTokenKind::RightBrace, Span::empty()));
            }
            ',' => {
                chars.next();
                tokens.push(YamlToken::new(YamlTokenKind::Comma, Span::empty()));
            }
            ':' => {
                chars.next();
                tokens.push(YamlToken::new(YamlTokenKind::Colon, Span::empty()));
            }
            '"' => {
                let scalar = parse_quoted(&mut chars, '"')?;
                tokens.push(YamlToken::new(YamlTokenKind::Scalar(scalar), Span::empty()));
            }
            '\'' => {
                let scalar = parse_quoted(&mut chars, '\'')?;
                tokens.push(YamlToken::new(YamlTokenKind::Scalar(scalar), Span::empty()));
            }
            _ => {
                let mut value = String::new();
                while let Some(ch) = chars.peek().copied() {
                    if ch.is_whitespace() || matches!(ch, '[' | ']' | '{' | '}' | ',' | ':') {
                        break;
                    }
                    value.push(ch);
                    chars.next();
                }
                let value = parse_scalar(&value)?;
                tokens.push(YamlToken::new(YamlTokenKind::Scalar(value), Span::empty()));
            }
        }
    }
    Ok(tokens)
}

fn parse_quoted<I>(chars: &mut std::iter::Peekable<I>, quote: char) -> Result<String>
where
    I: Iterator<Item = char>,
{
    let mut result = String::new();
    let _ = chars.next();
    while let Some(ch) = chars.next() {
        if ch == quote {
            return Ok(result);
        }
        if quote == '"' && ch == '\\' {
            let next = chars.next().ok_or_else(|| {
                Error::with_message(
                    ErrorKind::InvalidEscapeSequence,
                    Span::empty(),
                    "invalid escape".to_string(),
                )
            })?;
            match next {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidEscapeSequence,
                        Span::empty(),
                        "invalid escape".to_string(),
                    ));
                }
            }
        } else {
            result.push(ch);
        }
    }
    Err(Error::with_message(
        ErrorKind::UnterminatedString,
        Span::empty(),
        "unterminated string".to_string(),
    ))
}

fn split_key_value(line: &str) -> Result<Option<(&str, Option<&str>)>> {
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_escape = false;
    for (idx, ch) in line.char_indices() {
        match ch {
            '\\' if in_double => {
                prev_escape = !prev_escape;
                continue;
            }
            '"' if !in_single && !prev_escape => {
                in_double = !in_double;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            ':' if !in_single && !in_double => {
                let key = line[..idx].trim();
                let rest = line[idx + 1..].trim();
                let value = if rest.is_empty() { None } else { Some(rest) };
                return Ok(Some((key, value)));
            }
            _ => {}
        }
        prev_escape = false;
    }
    Ok(None)
}

fn parse_scalar(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        return parse_double_quoted(inner);
    }
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        return Ok(inner.replace("''", "'"));
    }
    Ok(trimmed.to_string())
}

fn parse_double_quoted(input: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let next = chars.next().ok_or_else(|| {
                Error::with_message(
                    ErrorKind::InvalidEscapeSequence,
                    Span::empty(),
                    "invalid escape".to_string(),
                )
            })?;
            match next {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => {
                    return Err(Error::with_message(
                        ErrorKind::InvalidEscapeSequence,
                        Span::empty(),
                        "invalid escape".to_string(),
                    ));
                }
            }
        } else {
            result.push(ch);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next_kind(lexer: &mut YamlLexer<'_>) -> Result<YamlTokenKind> {
        Ok(lexer.next_token()?.kind)
    }

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
    fn test_simple_mapping() -> Result<()> {
        let input = b"name: John\nage: 30\n";
        let mut lexer = YamlLexer::new(input);

        ensure_eq(
            next_kind(&mut lexer)?,
            YamlTokenKind::Scalar("name".to_string()),
        )?;
        ensure_eq(next_kind(&mut lexer)?, YamlTokenKind::Colon)?;
        ensure_eq(
            next_kind(&mut lexer)?,
            YamlTokenKind::Scalar("John".to_string()),
        )?;
        ensure_eq(next_kind(&mut lexer)?, YamlTokenKind::Newline)?;
        Ok(())
    }

    #[test]
    fn test_sequence() -> Result<()> {
        let input = b"- one\n- two\n";
        let mut lexer = YamlLexer::new(input);

        ensure_eq(next_kind(&mut lexer)?, YamlTokenKind::Dash)?;
        ensure_eq(
            next_kind(&mut lexer)?,
            YamlTokenKind::Scalar("one".to_string()),
        )?;
        ensure_eq(next_kind(&mut lexer)?, YamlTokenKind::Newline)?;
        Ok(())
    }
}
