//! JSON streaming parser events

use crate::value::Value;

/// Events emitted by the streaming JSON parser
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Start of a JSON object
    ObjectStart,
    /// End of a JSON object
    ObjectEnd,
    /// Start of a JSON array
    ArrayStart,
    /// End of a JSON array
    ArrayEnd,
    /// Object key (always followed by a value event)
    Key(String),
    /// JSON value (primitive or container start)
    Value(Value),
}
