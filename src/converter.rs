mod json_to_toml;
mod toml_to_json;

pub use json_to_toml::JsonToTomlConverter;
pub use toml_to_json::TomlToJsonConverter;

use crate::error::Result;
use crate::parser::value::Value;

pub struct Converter;

impl Converter {
    pub fn json_to_toml(json_value: Value) -> Result<Value> {
        JsonToTomlConverter::convert(json_value)
    }

    pub fn toml_to_json(toml_value: Value) -> Result<Value> {
        TomlToJsonConverter::convert(toml_value)
    }
}
