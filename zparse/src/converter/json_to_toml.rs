//! Converts JSON values to TOML format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of TOML restrictions

use crate::common::converter::{CommonConverter, ConversionContext};
use crate::error::{ConversionError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonToTomlConverter;

impl CommonConverter for JsonToTomlConverter {
    fn convert_map(map: HashMap<String, Value>, ctx: &mut ConversionContext) -> Result<Value> {
        let temp_map = Self::convert_map_inner(map, ctx)?;
        Ok(Value::Map(temp_map))
    }

    fn convert_array(arr: Vec<Value>, ctx: &mut ConversionContext) -> Result<Value> {
        // Check for nested null values in arrays
        if arr.iter().any(|v| matches!(v, Value::Null)) {
            let location = ctx.create_location();
            let path = ctx.get_path();
            return Err(location.create_error(
                ParseErrorKind::Conversion(ConversionError::UnsupportedValue(
                    "Null values in arrays".to_string(),
                )),
                &format!("Array at '{}' contains null values", path),
            ));
        }

        let converted = Self::convert_array_inner(arr, ctx)?;
        Ok(Value::Array(converted))
    }

    fn convert_value(value: Value, ctx: &mut ConversionContext) -> Result<Value> {
        match value {
            Value::Map(map) => Self::convert_map(map, ctx),
            Value::Array(arr) => Self::convert_array(arr, ctx),
            Value::Null => {
                let location = ctx.create_location();
                let path = ctx.get_path();
                let context = if path.is_empty() {
                    "Root level null value".to_string()
                } else {
                    format!("Null value at path: {}", path)
                };

                Err(location.create_error(
                    ParseErrorKind::Conversion(ConversionError::UnsupportedValue(
                        "Null value".to_string(),
                    )),
                    &context,
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
        let mut ctx = ConversionContext::new();
        Self::convert_map(map, &mut ctx)
    }
}
