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

impl Value {
    /// Creates a null value
    #[must_use]
    pub fn null() -> Self {
        Self::Null
    }

    /// Creates a boolean value
    #[must_use]
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Creates a number value
    #[must_use]
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Creates a string value
    #[must_use]
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Creates an array value
    #[must_use]
    pub fn array(values: Vec<Self>) -> Self {
        Self::Array(values)
    }

    /// Creates a map/object value
    #[must_use]
    pub fn map(entries: HashMap<String, Self>) -> Self {
        Self::Map(entries)
    }

    /// Creates a datetime value (TOML only)
    #[must_use]
    pub fn datetime(value: impl Into<String>) -> Self {
        Self::DateTime(value.into())
    }

    /// Returns true if this value is null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns true if this value is a boolean
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Returns true if this value is a number
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns true if this value is a string
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns true if this value is an array
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns true if this value is a map/object
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    /// Returns true if this value is a datetime
    #[must_use]
    pub fn is_datetime(&self) -> bool {
        matches!(self, Self::DateTime(_))
    }

    /// Returns the value as a boolean, if it is a boolean
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the value as a number, if it is a number
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns a reference to the string, if this value is a string
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to the array, if this value is an array
    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Returns a mutable reference to the array, if this value is an array
    #[must_use]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Returns a reference to the map, if this value is a map
    #[must_use]
    pub fn as_map(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a mutable reference to the map, if this value is a map
    #[must_use]
    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, Self>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a reference to the datetime string, if this value is a datetime
    #[must_use]
    pub fn as_datetime(&self) -> Option<&str> {
        match self {
            Self::DateTime(dt) => Some(dt),
            _ => None,
        }
    }
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

/// Helper function to compare values structurally rather than string representation
#[must_use]
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
