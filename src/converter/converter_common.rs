use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

/// Common trait for format converters
pub trait FormatConverter {
    /// Converts root value to target format
    fn convert_root(map: HashMap<String, Value>) -> Result<Value>;

    /// Converts array elements to target format
    fn convert_array_element(value: Value) -> Result<Value>;

    /// Common validation for root value
    fn validate_root(value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Object(map) | Value::Table(map) => Ok(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be an object/table".to_string(),
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
