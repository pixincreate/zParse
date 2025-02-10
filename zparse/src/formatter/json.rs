use std::collections::HashMap;

use super::{helpers, CommonFormatter, FormatConfig, Formatter};
use crate::{error::Result, parser::value::Value};

pub struct JsonFormatter;

impl CommonFormatter for JsonFormatter {}

impl Formatter for JsonFormatter {
    fn format(&self, value: &Value, config: &FormatConfig) -> Result<String> {
        Ok(Self::format_value(value, 0, config))?
    }
}

impl JsonFormatter {
    fn format_value(value: &Value, indent: usize, config: &FormatConfig) -> Result<String> {
        match value {
            Value::Array(arr) => Self::format_array(arr, indent, config),
            Value::Map(map) => Self::format_object(map, indent, config),
            _ => Ok(Self::format_basic_value(value)),
        }
    }

    fn format_array(arr: &[Value], indent: usize, config: &FormatConfig) -> Result<String> {
        if arr.is_empty() {
            return Ok(helpers::format_empty_array());
        }

        let (indent_str, inner_indent) = Self::create_indentation(indent, config)?;
        let mut items = Vec::new();

        for v in arr {
            let formatted = Self::format_value(v, indent + 1, config)?;
            items.push(format!("{}{}", inner_indent, formatted));
        }

        Ok(format!(
            "[\n{}\n{}]",
            helpers::join_with_commas(items),
            indent_str
        ))
    }

    fn format_object(
        map: &HashMap<String, Value>,
        indent: usize,
        config: &FormatConfig,
    ) -> Result<String> {
        if map.is_empty() {
            return Ok(helpers::format_empty_object());
        }

        let (indent_str, inner_indent) = Self::create_indentation(indent, config)?;
        let entries = Self::sort_entries(map.iter().collect(), config);

        let mut items = Vec::new();
        for (k, v) in entries {
            let formatted = Self::format_value(v, indent + 1, config)?;
            items.push(format!("{}\"{}\": {}", inner_indent, k, formatted));
        }

        Ok(format!(
            "{{\n{}\n{}}}",
            helpers::join_with_commas(items),
            indent_str
        ))
    }
}
