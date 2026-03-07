use std::fmt::Debug;
use zparse::error::{Error, ErrorKind, Result, Span};
use zparse::lexer::json::JsonLexer;
use zparse::lexer::token::TokenKind;

fn ensure_eq<T: PartialEq + Debug>(left: T, right: T) -> Result<()> {
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
fn test_lexer_structural_tokens() -> Result<()> {
    let input = b"{ } [ ] : ,";
    let mut lexer = JsonLexer::new(input);

    ensure_eq(lexer.next_token()?.kind, TokenKind::LeftBrace)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::RightBrace)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::LeftBracket)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::RightBracket)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Colon)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Comma)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Eof)?;
    Ok(())
}

#[test]
fn test_lexer_literals() -> Result<()> {
    let input = b"null true false";
    let mut lexer = JsonLexer::new(input);

    ensure_eq(lexer.next_token()?.kind, TokenKind::Null)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::True)?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::False)?;
    Ok(())
}

#[test]
fn test_lexer_string() -> Result<()> {
    let input = br#""hello world""#;
    let mut lexer = JsonLexer::new(input);

    ensure_eq(
        lexer.next_token()?.kind,
        TokenKind::String("hello world".to_string()),
    )?;
    Ok(())
}

#[test]
fn test_lexer_string_escapes() -> Result<()> {
    let input = br#""hello\nworld\t!\"\\\/\b\f""#;
    let mut lexer = JsonLexer::new(input);

    ensure_eq(
        lexer.next_token()?.kind,
        TokenKind::String("hello\nworld\t!\"\\/\x08\x0C".to_string()),
    )?;
    Ok(())
}

#[test]
fn test_lexer_string_unicode_escape() -> Result<()> {
    let input = br#""hello \u0041\u0042\u0043""#;
    let mut lexer = JsonLexer::new(input);

    ensure_eq(
        lexer.next_token()?.kind,
        TokenKind::String("hello ABC".to_string()),
    )?;
    Ok(())
}

#[test]
fn test_lexer_number_integer() -> Result<()> {
    let input = b"123 -456 0";
    let mut lexer = JsonLexer::new(input);

    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(123.0))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(-456.0))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(0.0))?;
    Ok(())
}

#[test]
fn test_lexer_number_fraction() -> Result<()> {
    let input = b"3.14 -0.5 123.456";
    let mut lexer = JsonLexer::new(input);

    let three_fourteen = 314_f64 / 100.0;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(three_fourteen))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(-0.5))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(123.456))?;
    Ok(())
}

#[test]
fn test_lexer_number_exponent() -> Result<()> {
    let input = b"1e10 1E10 1e+5 1e-5 3.14e-2";
    let mut lexer = JsonLexer::new(input);

    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(1e10))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(1E10))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(1e5))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(1e-5))?;
    let expected = "3.14e-2".parse::<f64>().map_err(|_| {
        Error::with_message(
            ErrorKind::InvalidNumber,
            Span::empty(),
            "failed to parse expected exponent".to_string(),
        )
    })?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(expected))?;
    Ok(())
}

#[test]
fn test_lexer_iterator() -> Result<()> {
    let input = b"[1, 2, 3]";
    let lexer = JsonLexer::new(input);
    let tokens: Result<Vec<_>> = lexer.map(|t| t.map(|token| token.kind)).collect();
    let tokens = tokens?;

    ensure_eq(
        tokens,
        vec![
            TokenKind::LeftBracket,
            TokenKind::Number(1.0),
            TokenKind::Comma,
            TokenKind::Number(2.0),
            TokenKind::Comma,
            TokenKind::Number(3.0),
            TokenKind::RightBracket,
        ],
    )?;
    Ok(())
}

#[test]
fn test_lexer_unterminated_string() {
    let input = br#""hello"#;
    let mut lexer = JsonLexer::new(input);

    let result = lexer.next_token();
    assert!(matches!(result, Err(err) if *err.kind() == ErrorKind::UnterminatedString));
}

#[test]
fn test_lexer_invalid_escape() {
    let input = br#""hello\x""#;
    let mut lexer = JsonLexer::new(input);

    let result = lexer.next_token();
    assert!(matches!(
        result,
        Err(err) if *err.kind() == ErrorKind::InvalidEscapeSequence
    ));
}

#[test]
fn test_lexer_invalid_number() -> Result<()> {
    let input = b"01";
    let mut lexer = JsonLexer::new(input);

    // This should parse as 0, then 1 as separate token
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(0.0))?;
    ensure_eq(lexer.next_token()?.kind, TokenKind::Number(1.0))?;
    Ok(())
}

#[test]
fn test_lexer_invalid_token() {
    let input = b"@";
    let mut lexer = JsonLexer::new(input);

    let result = lexer.next_token();
    assert!(matches!(result, Err(err) if *err.kind() == ErrorKind::InvalidToken));
}

#[test]
fn test_lexer_empty_string() -> Result<()> {
    let input = b"\"\"";
    let mut lexer = JsonLexer::new(input);

    ensure_eq(lexer.next_token()?.kind, TokenKind::String("".to_string()))?;
    Ok(())
}

#[test]
fn test_lexer_unicode_escape_invalid() {
    let input = br#""\u00GH""#;
    let mut lexer = JsonLexer::new(input);

    let result = lexer.next_token();
    assert!(matches!(
        result,
        Err(err) if *err.kind() == ErrorKind::InvalidUnicodeEscape
    ));
}

#[test]
fn test_lexer_whitespace() -> Result<()> {
    let input = b"  \t\n\r  null  ";
    let mut lexer = JsonLexer::new(input);

    let token = lexer.next_token()?;
    ensure_eq(token.kind, TokenKind::Null)?;
    ensure_eq(token.span.start.line, 2)?;
    Ok(())
}
