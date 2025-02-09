//! Converts JSON values to TOML format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of TOML restrictions

use crate::common::converter::CommonConverter;
use crate::error::{ConversionError, Location, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonToTomlConverter;

impl CommonConverter for JsonToTomlConverter {
    fn convert_map(map: HashMap<String, Value>) -> Result<Value> {
        let temp_map = Self::convert_map_inner(map)?;
        Ok(Value::Map(temp_map))
    }

    fn convert_array(arr: Vec<Value>) -> Result<Value> {
        // Check for nested null values in arrays
        if arr.iter().any(|v| matches!(v, Value::Null)) {
            let location = Location::new(0, 0);
            return Err(location.create_error(
                ParseErrorKind::Conversion(ConversionError::UnsupportedValue(
                    "Null values in arrays".to_string(),
                )),
                "TOML arrays cannot contain null values",
            ));
        }

        let converted = arr
            .into_iter()
            .map(Self::convert_value)
            .collect::<Result<Vec<_>>>()?;
        Ok(Value::Array(converted))
    }

    fn convert_value(value: Value) -> Result<Value> {
        match value {
            Value::Map(map) => Self::convert_map(map),
            Value::Array(arr) => Self::convert_array(arr),
            Value::Null => {
                let location = Location::new(0, 0);
                Err(location.create_error(
                    ParseErrorKind::Conversion(ConversionError::UnsupportedValue(
                        "Null value".to_string(),
                    )),
                    "TOML does not support null values",
                ))
            }
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
