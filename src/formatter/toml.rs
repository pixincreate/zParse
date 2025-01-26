use super::{FormatConfig, Formatter};
use crate::parser::Value;
use std::collections::HashMap;

pub struct TomlFormatter;

impl Formatter for TomlFormatter {
    fn format(&self, value: &Value, config: &FormatConfig) -> String {
        match value {
            Value::Table(map) => Self::format_table(map, vec![], config),
            _ => Value::to_string(value),
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
        let mut entries: Vec<_> = map.iter().collect();
        if config.sort_keys {
            entries.sort_by_key(|(k, _)| *k);
        }

        // Process simple key-value pairs first
        for (key, value) in entries.iter() {
            match value {
                Value::Table(_) | Value::Array(_) => continue,
                _ => {
                    result.push_str(&format!("{} = {}\n", key, value));
                }
            }
        }

        // Process tables and arrays
        for (key, value) in entries {
            match value {
                Value::Table(inner_map) => {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    let mut new_path = path.clone();
                    new_path.push(key.to_string());
                    result.push_str(&format!("[{}]\n", new_path.join(".")));
                    result.push_str(&Self::format_table(inner_map, new_path, config));
                }
                Value::Array(arr) if Self::is_table_array(arr.as_slice()) => {
                    // Changed here
                    for item in arr {
                        if let Value::Table(inner_map) = item {
                            let mut new_path = path.clone();
                            new_path.push(key.to_string());
                            result.push_str(&format!("\n[[{}]]\n", new_path.join(".")));
                            result.push_str(&Self::format_table(inner_map, new_path, config));
                        }
                    }
                }
                Value::Array(_) => {
                    result.push_str(&format!("{} = {}\n", key, value));
                }
                _ => {} // Already handled above
            }
        }

        result
    }

    fn is_table_array(arr: &[Value]) -> bool {
        // No change needed here
        arr.iter().any(|v| matches!(v, Value::Table(_)))
    }
}
