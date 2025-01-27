use super::{helpers, CommonFormatter, FormatConfig, Formatter};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonFormatter;

impl CommonFormatter for JsonFormatter {}

impl Formatter for JsonFormatter {
    fn format(&self, value: &Value, config: &FormatConfig) -> String {
        Self::format_value(value, 0, config)
    }
}

impl JsonFormatter {
    fn format_value(value: &Value, indent: usize, config: &FormatConfig) -> String {
        match value {
            Value::Array(arr) => Self::format_array(arr, indent, config),
            Value::Object(map) | Value::Table(map) => Self::format_object(map, indent, config),
            _ => Self::format_basic_value(value),
        }
    }

    fn format_array(arr: &[Value], indent: usize, config: &FormatConfig) -> String {
        if arr.is_empty() {
            return helpers::format_empty_array();
        }

        let (indent_str, inner_indent) = Self::create_indentation(indent, config);
        let items: Vec<String> = arr
            .iter()
            .map(|v| {
                format!(
                    "{}{}",
                    inner_indent,
                    Self::format_value(v, indent + 1, config)
                )
            })
            .collect();

        format!("[\n{}\n{}]", helpers::join_with_commas(items), indent_str)
    }

    fn format_object(map: &HashMap<String, Value>, indent: usize, config: &FormatConfig) -> String {
        if map.is_empty() {
            return helpers::format_empty_object();
        }

        let (indent_str, inner_indent) = Self::create_indentation(indent, config);
        let entries = Self::sort_entries(map.iter().collect(), config);

        let items: Vec<String> = entries
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}\"{}\": {}",
                    inner_indent,
                    k,
                    Self::format_value(v, indent + 1, config)
                )
            })
            .collect();

        format!("{{\n{}\n{}}}", helpers::join_with_commas(items), indent_str)
    }
}
