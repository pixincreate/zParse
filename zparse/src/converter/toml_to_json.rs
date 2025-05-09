use std::collections::HashMap;

use super::{CommonConverter, ConversionContext};
use crate::{error::Result, parser::value::Value};

pub struct TomlToJsonConverter;

impl CommonConverter for TomlToJsonConverter {
    fn convert_map(map: HashMap<String, Value>, ctx: &mut ConversionContext) -> Result<Value> {
        let json_map = Self::convert_map_inner(map, ctx).unwrap_or_default();
        Ok(Value::Map(json_map))
    }

    fn convert_array(arr: Vec<Value>, ctx: &mut ConversionContext) -> Result<Value> {
        let converted = Self::convert_array_inner(arr, ctx)?;
        Ok(Value::Array(converted))
    }

    fn convert_value(value: &Value, ctx: &mut ConversionContext) -> Result<Value> {
        match value {
            Value::Map(map) => Self::convert_map(map.clone(), ctx),
            Value::Array(arr) => Self::convert_array(arr.clone(), ctx),
            Value::Boolean(b) => Ok(Value::Boolean(*b)),
            Value::Number(n) => Ok(Value::Number(*n)),
            Value::String(s) => Ok(Value::String(s.clone())),
            Value::DateTime(dt) => Ok(Value::DateTime(dt.clone())),
            Value::Null => Ok(Value::Null),
        }
    }
}

impl TomlToJsonConverter {
    pub fn convert(value: &Value) -> Result<Value> {
        let map = Self::validate_root(value)?;
        let mut ctx = ConversionContext::new();
        Self::convert_map(map, &mut ctx)
    }
}
