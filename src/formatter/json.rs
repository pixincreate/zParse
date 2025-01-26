use super::{FormatConfig, Formatter};
use crate::parser::Value;
use std::collections::HashMap;

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, value: &Value, config: &FormatConfig) -> String {
        Self::format_value(value, 0, config)
    }
}

impl JsonFormatter {
    fn format_value(value: &Value, indent: usize, config: &FormatConfig) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::DateTime(dt) => format!("\"{}\"", dt),
            Value::Array(arr) => Self::format_array(arr, indent, config),
            Value::Object(map) | Value::Table(map) => Self::format_object(map, indent, config),
        }
    }

    fn format_array(arr: &[Value], indent: usize, config: &FormatConfig) -> String {
        if arr.is_empty() {
            return "[]".to_string();
        }

        let indent_str = " ".repeat(indent * config.indent_spaces);
        let inner_indent = " ".repeat((indent + 1) * config.indent_spaces);

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

        format!("[\n{}\n{}]", items.join(",\n"), indent_str)
    }

    fn format_object(map: &HashMap<String, Value>, indent: usize, config: &FormatConfig) -> String {
        if map.is_empty() {
            return "{}".to_string();
        }

        let indent_str = " ".repeat(indent * config.indent_spaces);
        let inner_indent = " ".repeat((indent + 1) * config.indent_spaces);

        let mut entries: Vec<_> = map.iter().collect();
        if config.sort_keys {
            entries.sort_by_key(|(k, _)| *k);
        }

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

        format!("{{\n{}\n{}}}", items.join(",\n"), indent_str)
    }
}
