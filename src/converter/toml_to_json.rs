use crate::common::converter::CommonConverter;
use crate::error::Result;
use crate::parser::Value;
use std::collections::HashMap;

pub struct TomlToJsonConverter;

impl CommonConverter for TomlToJsonConverter {
    fn convert_map(map: HashMap<String, Value>) -> Result<Value> {
        let mut json_map = HashMap::new();

        for (key, value) in map {
            let converted = Self::convert_value(value)?;
            json_map.insert(key, converted);
        }

        Ok(Value::Object(json_map))
    }

    fn convert_array(arr: Vec<Value>) -> Result<Value> {
        let converted = arr
            .into_iter()
            .map(Self::convert_value)
            .collect::<Result<Vec<_>>>()?;
        Ok(Value::Array(converted))
    }

    fn convert_value(value: Value) -> Result<Value> {
        match value {
            Value::Table(map) => Self::convert_map(map),
            Value::Array(arr) => Self::convert_array(arr),
            _ => Ok(value),
        }
    }
}

impl TomlToJsonConverter {
    pub fn convert(value: Value) -> Result<Value> {
        let map = Self::validate_root(value)?;
        Self::convert_map(map)
    }
}
