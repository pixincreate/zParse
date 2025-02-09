use crate::error::{Location, ParseErrorKind, Result, SemanticError};
use crate::parser::Value;
use std::collections::HashMap;

pub trait CommonConverter {
    fn convert_map(map: HashMap<String, Value>) -> Result<Value>;
    fn convert_array(arr: Vec<Value>) -> Result<Value>;
    fn convert_value(value: Value) -> Result<Value>;

    fn validate_root(value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Map(map) => Ok(map),
            _ => {
                let location = Location::new(0, 0); // Root level error
                Err(location.create_error(
                    ParseErrorKind::Semantic(SemanticError::TypeMismatch(
                        "Root must be an object/table".to_string(),
                    )),
                    "Invalid root value type",
                ))
            }
        }
    }

    fn convert_map_inner(map: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut new_map = HashMap::new();
        for (key, value) in map {
            let converted = Self::convert_value(value).map_err(|e| {
                let location = Location::new(0, 0);
                location.create_error(
                    e.kind().clone(),
                    &format!("Error converting value for key '{}'", key),
                )
            })?;
            new_map.insert(key, converted);
        }
        Ok(new_map)
    }
}
