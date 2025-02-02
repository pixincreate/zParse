mod json_to_toml;
mod toml_to_json;

pub use json_to_toml::JsonToTomlConverter;
pub use toml_to_json::TomlToJsonConverter;

use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

/// High-level “Converter” utility struct that delegates to the specialized
/// implementations for JSON↔TOML conversions.
pub struct Converter;

impl Converter {
    pub fn json_to_toml(json_value: Value) -> Result<Value> {
        JsonToTomlConverter::convert(json_value)
    }

    pub fn toml_to_json(toml_value: Value) -> Result<Value> {
        TomlToJsonConverter::convert(toml_value)
    }
}

/// Common trait for format converters
pub trait FormatConverter {
    /// Converts root value to target format
    fn convert_root(map: HashMap<String, Value>) -> Result<Value>;

    /// Converts array elements to target format
    fn convert_array_element(value: Value) -> Result<Value>;

    /// Common validation for root value
    fn validate_root(value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Map(map) => Ok(map),
            _ => Err(ParseError::new(ParseErrorKind::Syntax(
                crate::error::SyntaxError::InvalidValue("Root must be an object/table".to_string()),
            ))),
        }
    }

    /// Common array conversion logic
    fn convert_array(arr: Vec<Value>) -> Result<Value> {
        let converted = arr
            .into_iter()
            .map(Self::convert_array_element)
            .collect::<Result<Vec<_>>>()?;
        Ok(Value::Array(converted))
    }
}
