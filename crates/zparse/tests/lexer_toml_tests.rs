use zparse::error::{Error, ErrorKind, Result};
use zparse::lexer::toml::{TomlLexer, TomlTokenKind};

#[test]
fn test_simple_tokens() -> Result<()> {
    let input = b"[table]\nkey = 1\n";
    let mut lexer = TomlLexer::new(input);

    matches_token(&mut lexer, TomlTokenKind::LeftBracket)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("table".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::RightBracket)?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("key".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::Integer(1))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    Ok(())
}

#[test]
fn test_string_tokens() -> Result<()> {
    let input = b"title = \"hello\"\nname = 'world'\n";
    let mut lexer = TomlLexer::new(input);

    matches_token(&mut lexer, TomlTokenKind::BareKey("title".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::String("hello".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("name".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::String("world".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    Ok(())
}

#[test]
fn test_numbers_and_bool() -> Result<()> {
    let input = b"flag = true\nint = -42\nfloat = 3.5\n";
    let mut lexer = TomlLexer::new(input);

    matches_token(&mut lexer, TomlTokenKind::BareKey("flag".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::Bool(true))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("int".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::Integer(-42))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("float".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::Float(3.5))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    Ok(())
}

#[test]
fn test_array_table_tokens() -> Result<()> {
    let input = b"[[products]]\nname = \"book\"\n";
    let mut lexer = TomlLexer::new(input);

    matches_token(&mut lexer, TomlTokenKind::DoubleLeftBracket)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("products".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::DoubleRightBracket)?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    matches_token(&mut lexer, TomlTokenKind::BareKey("name".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(&mut lexer, TomlTokenKind::String("book".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    Ok(())
}

#[test]
fn test_datetime_token() -> Result<()> {
    let input = b"date = 1979-05-27T07:32:00Z\n";
    let mut lexer = TomlLexer::new(input);

    matches_token(&mut lexer, TomlTokenKind::BareKey("date".to_string()))?;
    matches_token(&mut lexer, TomlTokenKind::Equals)?;
    matches_token(
        &mut lexer,
        TomlTokenKind::Datetime("1979-05-27T07:32:00Z".to_string()),
    )?;
    matches_token(&mut lexer, TomlTokenKind::Newline)?;
    Ok(())
}

fn matches_token(lexer: &mut TomlLexer<'_>, expected: TomlTokenKind) -> Result<()> {
    let token = lexer.next_token()?;
    if token.kind != expected {
        return Err(Error::with_message(
            ErrorKind::InvalidToken,
            token.span,
            format!("expected {expected:?}, got {actual:?}", actual = token.kind),
        ));
    }
    Ok(())
}
