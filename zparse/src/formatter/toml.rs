use std::collections::HashMap;

use super::{CommonFormatter, FormatConfig, Formatter};
use crate::{error::Result, parser::value::Value};

pub struct TomlFormatter;

impl CommonFormatter for TomlFormatter {}

impl Formatter for TomlFormatter {
    fn format(&self, value: &Value, config: &FormatConfig) -> Result<String> {
        match value {
            Value::Map(map) => Ok(Self::format_table(map, vec![], config)),
            _ => Ok(Value::to_string(value)),
        }
    }
}

impl TomlFormatter {
    fn format_table(
        map: &HashMap<String, Value>,
        path: Vec<String>,
        config: &FormatConfig,
    ) -> String {
        let mut result = String::new();
        let entries = Self::sort_entries(map.iter().collect(), config);

        // Process simple key-value pairs first
        for (key, value) in entries.iter() {
            match value {
                Value::Map(_) | Value::Array(_) => continue,
                _ => {
                    result.push_str(&format!("{} = {}\n", key, Self::format_basic_value(value)));
                }
            }
        }

        // Process tables and arrays
        for (key, value) in entries {
            match value {
                Value::Map(inner_map) => {
                    Self::format_regular_table(&mut result, inner_map, key, path.clone(), config);
                }
                Value::Array(arr) if Self::is_table_array(arr) => {
                    Self::format_array_table(&mut result, arr, key, path.clone(), config);
                }
                Value::Array(_) => {
                    result.push_str(&format!("{} = {}\n", key, value));
                }
                _ => {} // Already handled above
            }
        }

        result
    }

    fn format_regular_table(
        result: &mut String,
        inner_map: &HashMap<String, Value>,
        key: &str,
        mut path: Vec<String>,
        config: &FormatConfig,
    ) {
        if !result.is_empty() {
            result.push('\n');
        }
        path.push(key.to_string());
        result.push_str(&format!("[{}]\n", path.join(".")));
        result.push_str(&Self::format_table(inner_map, path, config));
    }

    fn format_array_table(
        result: &mut String,
        arr: &[Value],
        key: &str,
        mut path: Vec<String>,
        config: &FormatConfig,
    ) {
        for item in arr {
            if let Value::Map(inner_map) = item {
                path.push(key.to_string());
                result.push_str(&format!("\n[[{}]]\n", path.join(".")));
                result.push_str(&Self::format_table(inner_map, path.clone(), config));
                path.pop();
            }
        }
    }
}
