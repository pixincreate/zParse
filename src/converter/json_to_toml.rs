use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonToTomlConverter;

impl JsonToTomlConverter {
    pub fn convert(json_value: Value) -> Result<Value> {
        match json_value {
            Value::Object(map) => Self::convert_object(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be an object".to_string(),
            ))),
        }
    }

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
