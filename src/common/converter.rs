use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub trait CommonConverter {
    fn convert_map(map: HashMap<String, Value>) -> Result<Value>;
    fn convert_array(arr: Vec<Value>) -> Result<Value>;
    fn convert_value(value: Value) -> Result<Value>;

    fn validate_root(value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Map(map) => Ok(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be an object/table".to_string(),
            ))),
        }
    }

    fn convert_map_inner(map: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut new_map = HashMap::new();
        for (key, value) in map {
            let converted = Self::convert_value(value)?;
            new_map.insert(key, converted);
        }
        Ok(new_map)
    }
}
