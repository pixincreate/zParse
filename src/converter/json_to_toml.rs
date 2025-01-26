//! Converts JSON values to TOML format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of TOML restrictions

use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonToTomlConverter;

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
        match json_value {
            Value::Object(map) => Self::convert_object(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be an object".to_string(),
            ))),
        }
    }

    /// Converts a JSON object to a TOML table
    fn convert_object(map: HashMap<String, Value>) -> Result<Value> {
        let mut toml_map = HashMap::new();

        for (key, value) in map {
            match value {
                Value::Object(inner_map) => {
                    // Convert nested objects to TOML tables
                    toml_map.insert(key, Self::convert_object(inner_map)?);
                }
                Value::Array(arr) => {
                    // Convert array elements
                    let converted = arr
                        .into_iter()
                        .map(Self::convert_array_element)
                        .collect::<Result<Vec<_>>>()?;
                    toml_map.insert(key, Value::Array(converted));
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

    /// Converts an array element to TOML format
    fn convert_array_element(value: Value) -> Result<Value> {
        match value {
            Value::Object(map) => Self::convert_object(map),
            Value::Array(arr) => {
                let converted = arr
                    .into_iter()
                    .map(Self::convert_array_element)
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(converted))
            }
            Value::Null => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "TOML does not support null".to_string(),
            ))),
            _ => Ok(value),
        }
    }
}
