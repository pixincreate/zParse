use crate::{
    error::{ParseError, ParseErrorKind, Result},
    formatter::{FormatConfig, Formatter, JsonFormatter, TomlFormatter},
    parser::{json::JsonParser, toml::TomlParser, value::Value},
};
use std::fs;

pub fn read_file(path: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|_| {
        ParseError::new(ParseErrorKind::IoError(format!(
            "Cannot read file: {}",
            path
        )))
    })
}

pub fn write_file(path: &str, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|_| {
        ParseError::new(ParseErrorKind::IoError(format!(
            "Cannot write to file: {}",
            path
        )))
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

pub fn format_json(value: &Value) -> String {
    JsonFormatter.format(value, &FormatConfig::default())
}

pub fn format_toml(value: &Value) -> String {
    TomlFormatter.format(value, &FormatConfig::default())
}
