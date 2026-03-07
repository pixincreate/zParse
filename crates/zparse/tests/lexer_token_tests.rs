use zparse::error::{Pos, Span};
use zparse::lexer::token::{Token, TokenKind};

#[test]
fn test_token_kind_name() {
    assert_eq!(TokenKind::LeftBrace.name(), "'{'");
    assert_eq!(TokenKind::Null.name(), "null");
    assert_eq!(TokenKind::String("test".to_string()).name(), "string");
}

#[test]
fn test_token_kind_is_value() {
    assert!(TokenKind::Null.is_value());
    assert!(TokenKind::True.is_value());
    assert!(TokenKind::String("x".to_string()).is_value());
    assert!(TokenKind::Number(42.0).is_value());
    assert!(TokenKind::LeftBrace.is_value());
    assert!(TokenKind::LeftBracket.is_value());
    assert!(!TokenKind::Comma.is_value());
    assert!(!TokenKind::Colon.is_value());
}

#[test]
fn test_token_creation() {
    let span = Span::new(Pos::new(0, 1, 1), Pos::new(4, 1, 5));
    let token = Token::new(TokenKind::Null, span);
    assert_eq!(token.kind, TokenKind::Null);
}
