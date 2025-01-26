//! Converts TOML values to JSON format.
//!
//! Handles the complexities of converting between formats including:
//! - Structural differences between JSON and TOML
//! - Type mapping between formats
//! - Validation of JSON restrictions

use super::converter_common::FormatConverter;
use crate::error::Result;
use crate::parser::Value;
use std::collections::HashMap;

pub struct TomlToJsonConverter;

impl FormatConverter for TomlToJsonConverter {
    fn convert_root(map: HashMap<String, Value>) -> Result<Value> {
        let mut json_map = HashMap::new();

        for (key, value) in map {
            json_map.insert(key, Self::convert_array_element(value)?);
        }

        Ok(Value::Object(json_map))
    }

    fn convert_array_element(value: Value) -> Result<Value> {
        Ok(match value {
            Value::Table(map) => Value::Object(map),
            Value::Array(arr) => Self::convert_array(arr)?,
            _ => value,
        })
    }
}

impl TomlToJsonConverter {
    pub fn convert(toml_value: Value) -> Result<Value> {
        let map = Self::validate_root(toml_value)?;
        Self::convert_root(map)
    }
}
