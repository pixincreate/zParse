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
