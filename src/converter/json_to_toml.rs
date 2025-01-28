//! Converts JSON values to TOML format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of TOML restrictions
//!
use crate::common::converter::CommonConverter;
use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonToTomlConverter;

impl CommonConverter for JsonToTomlConverter {
    fn convert_map(map: HashMap<String, Value>) -> Result<Value> {
        let mut toml_map = HashMap::new();

        for (key, value) in map {
            let converted = Self::convert_value(value)?;
            toml_map.insert(key, converted);
        }

        Ok(Value::Table(toml_map))
    }

    fn convert_array(arr: Vec<Value>) -> Result<Value> {
        let converted = arr
            .into_iter()
            .map(Self::convert_value)
            .collect::<Result<Vec<_>>>()?;
        Ok(Value::Array(converted))
    }

    fn convert_value(value: Value) -> Result<Value> {
        match value {
            Value::Object(map) => Self::convert_map(map),
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
    pub fn convert(value: Value) -> Result<Value> {
        let map = Self::validate_root(value)?;
        Self::convert_map(map)
    }
}
