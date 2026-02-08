//! YAML streaming parser events

use crate::value::Value;

/// YAML parser events
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    MappingStart,
    MappingEnd,
    SequenceStart,
    SequenceEnd,
    Key(String),
    Value(Value),
}
