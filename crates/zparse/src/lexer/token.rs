//! Token types for JSON lexer

use crate::error::Span;

/// JSON token types
#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // Structural
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Comma,        // ,

    // Literals
    Null,
    True,
    False,

    // Values
    String(String),
    Number(f64),

    // Special
    Eof,
}

impl TokenKind {
    /// Get token name for error messages
    pub const fn name(&self) -> &'static str {
        match self {
            Self::LeftBrace => "'{'",
            Self::RightBrace => "'}'",
            Self::LeftBracket => "'['",
            Self::RightBracket => "']'",
            Self::Colon => "':'",
            Self::Comma => "','",
            Self::Null => "null",
            Self::True => "true",
            Self::False => "false",
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Eof => "EOF",
        }
    }

    /// Check if token is a value (can appear in value position)
    pub const fn is_value(&self) -> bool {
        matches!(
            self,
            Self::Null
                | Self::True
                | Self::False
                | Self::String(_)
                | Self::Number(_)
                | Self::LeftBrace
                | Self::LeftBracket
        )
    }
}

/// Token with source location
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn eof(span: Span) -> Self {
        Self {
            kind: TokenKind::Eof,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Pos, Span};

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
}
