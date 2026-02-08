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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Array, Object};

    #[test]
    fn test_event_creation() {
        let events = vec![
            Event::ObjectStart,
            Event::ObjectEnd,
            Event::ArrayStart,
            Event::ArrayEnd,
            Event::Key("test".to_string()),
            Event::Value(Value::Null),
            Event::Value(Value::Bool(true)),
            Event::Value(Value::Number(42.0)),
            Event::Value(Value::String("hello".to_string())),
            Event::Value(Value::Array(Array::new())),
            Event::Value(Value::Object(Object::new())),
        ];

        assert_eq!(events.len(), 11);
    }

    #[test]
    fn test_event_equality() {
        assert_eq!(Event::ObjectStart, Event::ObjectStart);
        assert_eq!(Event::ArrayEnd, Event::ArrayEnd);
        assert_eq!(
            Event::Key("test".to_string()),
            Event::Key("test".to_string())
        );
        assert_eq!(Event::Value(Value::Null), Event::Value(Value::Null));
        assert_ne!(Event::ObjectStart, Event::ObjectEnd);
        assert_ne!(Event::Value(Value::Null), Event::Value(Value::Bool(true)));
    }
}
