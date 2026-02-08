//! TOML streaming parser events

use crate::value::Value;

/// TOML parser event
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Start of a table or array-of-tables
    TableStart { path: Vec<String>, is_array: bool },
    /// Key/value pair within current table
    KeyValue { key: Vec<String>, value: Value },
}
