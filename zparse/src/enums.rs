use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Json,
    Toml,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Comma,        // ,
    Equals,       // =
    Dot,          // .
    String(String),
    Number(f64),
    Boolean(bool),
    DateTime(String),
    Null,
    EOF,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Equals => write!(f, "="),
            Self::Dot => write!(f, "."),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::DateTime(dt) => write!(f, "{}", dt),
            Self::Null => write!(f, "null"),
            Self::EOF => write!(f, "EOF"),
        }
    }
}
