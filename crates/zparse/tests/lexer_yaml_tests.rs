use zparse::error::{Error, ErrorKind, Result, Span};
use zparse::lexer::yaml::{YamlLexer, YamlTokenKind};

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
