//! Converts JSON values to TOML format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of TOML restrictions

use super::converter_common::FormatConverter;
use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonToTomlConverter;
impl FormatConverter for JsonToTomlConverter {
    fn convert_root(map: HashMap<String, Value>) -> Result<Value> {
        let mut toml_map = HashMap::new();

        for (key, value) in map {
            match value {
                Value::Object(inner_map) => {
                    toml_map.insert(key, Self::convert_root(inner_map)?);
                }
                Value::Array(arr) => {
                    toml_map.insert(key, Self::convert_array(arr)?);
                }
                Value::Null => {
                    return Err(ParseError::new(ParseErrorKind::InvalidValue(
                        "TOML does not support null".to_string(),
                    )))
                }
                _ => {
                    toml_map.insert(key, value);
                }
            }
        }

        Ok(Value::Table(toml_map))
    }

    fn convert_array_element(value: Value) -> Result<Value> {
        match value {
            Value::Object(map) => Self::convert_root(map),
            Value::Array(arr) => Self::convert_array(arr),
            Value::Null => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "TOML does not support null".to_string(),
            ))),
            _ => Ok(value),
        }
    }
}

impl JsonToTomlConverter {
    /// Converts a JSON value to TOML format
    ///
    /// # Arguments
    /// * `json_value` - The JSON value to convert
    ///
    /// # Returns
    /// * `Ok(Value)` - The converted TOML value
    /// * `Err` - If the JSON structure cannot be represented in TOML
    pub fn convert(json_value: Value) -> Result<Value> {
        let map = Self::validate_root(json_value)?;
        Self::convert_root(map)
    }
}
