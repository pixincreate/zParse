use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub trait CommonConverter {
    fn convert_map(map: HashMap<String, Value>) -> Result<Value>;
    fn convert_array(arr: Vec<Value>) -> Result<Value>;

    fn validate_root(value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Object(map) | Value::Table(map) => Ok(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be an object/table".to_string(),
            ))),
        }
    }

    fn convert_value(value: Value) -> Result<Value>;
}
