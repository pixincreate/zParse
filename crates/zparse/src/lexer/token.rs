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
