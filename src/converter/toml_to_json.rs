//! Converts TOML values to JSON format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of JSON restrictions

use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct TomlToJsonConverter;

impl TomlToJsonConverter {
    /// Converts a TOML value to JSON format
    ///
    /// # Arguments
    /// * `toml_value` - The TOML value to convert
    ///
    /// # Returns
    /// * `Ok(Value)` - The converted JSON value
    /// * `Err` - If the TOML structure cannot be represented in JSON
    pub fn convert(toml_value: Value) -> Result<Value> {
        match toml_value {
            Value::Table(map) | Value::Object(map) => Self::convert_table(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be a table".to_string(),
            ))),
        }
    }

    /// Converts a TOML table to a JSON object
    fn convert_table(map: HashMap<String, Value>) -> Result<Value> {
        let mut json_map = HashMap::new();

        for (key, value) in map {
            json_map.insert(key, Self::convert_value(value)?);
        }

        Ok(Value::Object(json_map))
    }

    /// Converts a TOML value to a JSON value
    fn convert_value(value: Value) -> Result<Value> {
        Ok(match value {
            Value::Table(map) => Value::Object(map),
            Value::Array(arr) => {
                let mut json_arr = Vec::new();
                for item in arr {
                    json_arr.push(Self::convert_value(item)?);
                }
                Value::Array(json_arr)
            }
            _ => value,
        })
    }
}
