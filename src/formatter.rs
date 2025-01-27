mod json;
mod toml;

pub use self::{json::JsonFormatter, toml::TomlFormatter};
use crate::parser::Value;

/// Configuration options for formatting
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Number of spaces for indentation
    pub indent_spaces: usize,
    /// Whether to sort object keys
    pub sort_keys: bool,
}

/// Default configuration for formatting
impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_spaces: 2,
            sort_keys: true,
        }
    }
}

/// Trait for formatting a Value as a string
pub trait Formatter {
    fn format(&self, value: &Value, config: &FormatConfig) -> String;
}

/// Common formatting functionality shared between JSON and TOML formatters
pub trait CommonFormatter {
    /// Formats basic values (null, bool, number, string, datetime)
    fn format_basic_value(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::DateTime(dt) => format!("\"{}\"", dt),
            Value::Array(_) => helpers::format_empty_array(),
            Value::Object(_) | Value::Table(_) => helpers::format_empty_object(),
        }
    }

    /// Creates indentation strings
    fn create_indentation(indent: usize, config: &FormatConfig) -> (String, String) {
        let indent_str = " ".repeat(indent * config.indent_spaces);
        let inner_indent = " ".repeat((indent + 1) * config.indent_spaces);
        (indent_str, inner_indent)
    }

    /// Sorts entries if configured
    fn sort_entries<'a>(
        entries: Vec<(&'a String, &'a Value)>,
        config: &FormatConfig,
    ) -> Vec<(&'a String, &'a Value)> {
        let mut entries = entries;
        if config.sort_keys {
            entries.sort_by_key(|(k, _)| *k);
        }
        entries
    }

    /// Checks if an array contains tables
    fn is_table_array(arr: &[Value]) -> bool {
        arr.iter().any(|v| matches!(v, Value::Table(_)))
    }
}

/// Helper functions for formatting collections
pub mod helpers {
    pub fn format_empty_array() -> String {
        "[]".to_string()
    }

    pub fn format_empty_object() -> String {
        "{}".to_string()
    }

    pub fn join_with_commas(items: Vec<String>) -> String {
        items.join(",\n")
    }
}
