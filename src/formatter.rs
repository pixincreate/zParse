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
