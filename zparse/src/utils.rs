use std::fs;

use crate::{
    error::{ParseError, Result},
    formatter::{FormatConfig, Formatter, JsonFormatter, TomlFormatter},
    parser::{json::JsonParser, toml::TomlParser, value::Value},
};

pub fn read_file(path: &str) -> Result<String> {
    let path_copy = path.to_string(); // Keep a copy for context
    fs::read_to_string(path).map_err(|e| {
        let mut err: ParseError = e.into();
        err = err.with_context(format!("Error reading file: {}", path_copy));
        err
    })
}

pub fn write_file(path: &str, content: &str) -> Result<()> {
    let path_copy = path.to_string(); // Keep a copy for context
    fs::write(path, content).map_err(|e| {
        let mut err: ParseError = e.into();
        err = err.with_context(format!("Error writing to file: {}", path_copy));
        err
    })
}

pub fn parse_json(content: &str) -> Result<Value> {
    let mut parser = JsonParser::new(content)?;
    parser.parse()
}

pub fn parse_toml(content: &str) -> Result<Value> {
    let mut parser = TomlParser::new(content)?;
    parser.parse()
}

pub fn format_json(value: &Value) -> Result<String> {
    JsonFormatter.format(value, &FormatConfig::default())
}

pub fn format_toml(value: &Value) -> Result<String> {
    TomlFormatter.format(value, &FormatConfig::default())
}
