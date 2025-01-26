use crate::error::{ParseError, ParseErrorKind, Result};
use crate::parser::Value;
use std::collections::HashMap;

pub struct TomlToJsonConverter;

impl TomlToJsonConverter {
    pub fn convert(toml_value: Value) -> Result<Value> {
        match toml_value {
            Value::Table(map) | Value::Object(map) => Self::convert_table(map),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Root must be a table".to_string(),
            ))),
        }
    }

    fn convert_table(map: HashMap<String, Value>) -> Result<Value> {
        let mut json_map = HashMap::new();

        for (key, value) in map {
            json_map.insert(key, Self::convert_value(value)?);
        }

        Ok(Value::Object(json_map))
    }

    fn convert_value(value: Value) -> Result<Value> {
        Ok(match value {
            Value::Table(map) => Value::Object(map),
            Value::Array(arr) => {
                let mut json_arr = Vec::new();
                for item in arr {
                    json_arr.push(Self::convert_value(item)?);
                }
                Value::Array(json_arr)
            }
            _ => value,
        })
    }
}
