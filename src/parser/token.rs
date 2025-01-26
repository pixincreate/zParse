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
