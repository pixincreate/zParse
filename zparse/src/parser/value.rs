//! Value types that can represent both JSON and TOML data.
//!
//! This module defines a unified Value enum that can represent data from
//! either format, enabling conversion between them.

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
    Map(HashMap<String, Value>),
    /// Represents a TOML datetime
    DateTime(String),
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
            Self::Map(obj) => {
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

// Helper function to compare values structurally rather than string representation
pub fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Map(l_map), Value::Map(r_map)) => {
            if l_map.len() != r_map.len() {
                return false;
            }
            l_map
                .iter()
                .all(|(k, v)| r_map.get(k).is_some_and(|r_v| values_equal(v, r_v)))
        }
        (Value::Array(l_arr), Value::Array(r_arr)) => {
            if l_arr.len() != r_arr.len() {
                return false;
            }
            l_arr
                .iter()
                .zip(r_arr.iter())
                .all(|(l, r)| values_equal(l, r))
        }
        (Value::Number(l), Value::Number(r)) => (l - r).abs() < f64::EPSILON,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Null, Value::Null) => true,
        (Value::DateTime(l), Value::DateTime(r)) => l == r,
        _ => false,
    }
}
