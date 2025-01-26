use std::collections::HashMap;
use std::fmt;

/// Represents a parsed value that can be either JSON or TOML data.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represents a null value (JSON only)
    Null,
    /// Represents a boolean value
    Boolean(bool),
    /// Represents a number (stored as f64 for simplicity)
    Number(f64),
    /// Represents a string value
    String(String),
    /// Represents an array of values
    Array(Vec<Value>),
    /// Represents a JSON object or TOML table
    Object(HashMap<String, Value>),
    /// Represents a TOML datetime
    DateTime(String),
    /// Represents a TOML table
    Table(HashMap<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::DateTime(dt) => write!(f, "{}", dt),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Self::Object(obj) | Self::Table(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}
