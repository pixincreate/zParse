//! Property-based tests for JSON parsing
//!
//! These tests use proptest to verify:
//! 1. Roundtrip property: parse(value) -> serialize -> parse == original
//! 2. Valid JSON never panics: any valid JSON parses without error

use proptest::prelude::*;
use zparse::{from_str, Value};

/// Serialize a Value to JSON string
fn serialize_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // Handle special float values
            if n.is_nan() {
                "null".to_string()
            } else if n.is_infinite() {
                if *n > 0.0 {
                    "null".to_string()
                } else {
                    "null".to_string()
                }
            } else if *n == 0.0 && n.is_sign_negative() {
                "-0.0".to_string()
            } else if n.fract() == 0.0 {
                // Integer value
                format!("{:.0}", n)
            } else {
                format!("{}", n)
            }
        }
        Value::String(s) => format!("\"{}\"", escape_string(s)),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(serialize_value).collect();
            format!("[{}]", elements.join(","))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", escape_string(k), serialize_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
    }
}

/// Escape special characters in a string for JSON
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\x08' => result.push_str("\\b"), // backspace
            '\x0C' => result.push_str("\\f"), // form feed
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Strategy for generating arbitrary JSON strings (keys)
fn arb_json_string() -> impl Strategy<Value = String> {
    // Generate strings with alphanumeric characters and common special chars
    "[a-zA-Z0-9_]*".prop_map(|s| s)
}

/// Strategy for generating arbitrary JSON values
fn arb_json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        // Use reasonable numeric values to avoid float precision issues
        (-1e6f64..1e6f64)
            .prop_filter("Non-finite f64", |f| f.is_finite())
            .prop_map(Value::Number),
        arb_json_string().prop_map(Value::String),
    ];

    leaf.prop_recursive(8, 256, 10, |inner| {
        prop_oneof![
            // Generate arrays with 0-10 elements
            prop::collection::vec(inner.clone(), 0..10).prop_map(|v| Value::Array(v.into())),
            // Generate objects with 0-10 key-value pairs
            prop::collection::hash_map(arb_json_string(), inner, 0..10)
                .prop_map(|m| { Value::Object(m.into_iter().collect()) }),
        ]
    })
}

proptest! {
    /// Test that parsing then serializing then parsing returns the original value
    #[test]
    fn json_roundtrip(value in arb_json_value()) {
        let serialized = serialize_value(&value);
        let parsed = from_str(&serialized).unwrap();

        // Compare the parsed value with the original
        // We need to handle float comparison specially
        assert_values_equal(&parsed, &value);
    }

    /// Test that any valid JSON parses without panicking
    #[test]
    fn valid_json_parses(s in r#"\{(("[a-z0-9]+":[0-9]+)(,("[a-z0-9]+":[0-9]+))*)?\}"#) {
        // This generates simple valid JSON patterns
        let _result = from_str(&s);
        // Should not panic - we don't care about the result
    }

    /// Test that arrays roundtrip correctly
    #[test]
    fn array_roundtrip(arr in prop::collection::vec(arb_json_value(), 0..20)) {
        let value = Value::Array(arr.into());
        let serialized = serialize_value(&value);
        let parsed = from_str(&serialized).unwrap();
        assert_values_equal(&parsed, &value);
    }

    /// Test that objects roundtrip correctly
    #[test]
    fn object_roundtrip(obj in prop::collection::hash_map(arb_json_string(), arb_json_value(), 0..20)) {
        let value: Value = obj.into_iter().collect::<zparse::Object>().into();
        let serialized = serialize_value(&value);
        let parsed = from_str(&serialized).unwrap();
        assert_values_equal(&parsed, &value);
    }
}

/// Compare two values, handling float comparisons with tolerance
fn assert_values_equal(a: &Value, b: &Value) {
    match (a, b) {
        (Value::Null, Value::Null) => {}
        (Value::Bool(a1), Value::Bool(b1)) => assert_eq!(a1, b1),
        (Value::Number(a1), Value::Number(b1)) => {
            // Use relative tolerance for float comparison
            if (a1 - b1).abs() > 1e-10 * a1.abs().max(b1.abs()).max(1.0) {
                panic!("Numbers not equal: {} vs {}", a1, b1);
            }
        }
        (Value::String(a1), Value::String(b1)) => assert_eq!(a1, b1),
        (Value::Array(a1), Value::Array(b1)) => {
            assert_eq!(a1.len(), b1.len(), "Array lengths differ");
            for (_i, (ae, be)) in a1.iter().zip(b1.iter()).enumerate() {
                assert_values_equal(ae, be);
            }
        }
        (Value::Object(a1), Value::Object(b1)) => {
            assert_eq!(a1.len(), b1.len(), "Object lengths differ");
            for (key, a_val) in a1.iter() {
                let b_val = b1
                    .get(key)
                    .expect(&format!("Key '{}' missing in second object", key));
                assert_values_equal(a_val, b_val);
            }
        }
        _ => panic!("Value types differ: {:?} vs {:?}", a, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_null() {
        assert_eq!(serialize_value(&Value::Null), "null");
    }

    #[test]
    fn test_serialize_bool() {
        assert_eq!(serialize_value(&Value::Bool(true)), "true");
        assert_eq!(serialize_value(&Value::Bool(false)), "false");
    }

    #[test]
    fn test_serialize_number() {
        assert_eq!(serialize_value(&Value::Number(42.0)), "42");
        assert_eq!(serialize_value(&Value::Number(3.14)), "3.14");
        assert_eq!(serialize_value(&Value::Number(-123.0)), "-123");
    }

    #[test]
    fn test_serialize_string() {
        assert_eq!(
            serialize_value(&Value::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(
            serialize_value(&Value::String("hello world".to_string())),
            "\"hello world\""
        );
    }

    #[test]
    fn test_serialize_string_escaping() {
        assert_eq!(
            serialize_value(&Value::String("hello\nworld".to_string())),
            "\"hello\\nworld\""
        );
        assert_eq!(
            serialize_value(&Value::String("hello\"world\"".to_string())),
            "\"hello\\\"world\\\"\""
        );
        assert_eq!(
            serialize_value(&Value::String("hello\\world".to_string())),
            "\"hello\\\\world\""
        );
    }

    #[test]
    fn test_serialize_array() {
        let arr = Value::Array(vec![Value::Null, Value::Bool(true), Value::Number(42.0)].into());
        assert_eq!(serialize_value(&arr), "[null,true,42]");
    }

    #[test]
    fn test_serialize_object() {
        use zparse::Object;
        let mut obj = Object::new();
        obj.insert("name", Value::String("test".to_string()));
        obj.insert("value", Value::Number(123.0));
        assert_eq!(
            serialize_value(&Value::Object(obj)),
            "{\"name\":\"test\",\"value\":123}"
        );
    }

    #[test]
    fn test_simple_roundtrip() {
        let json = r#"{"name": "test", "value": 123}"#;
        let parsed = from_str(json).unwrap();
        let serialized = serialize_value(&parsed);
        let reparsed = from_str(&serialized).unwrap();
        assert_values_equal(&parsed, &reparsed);
    }

    #[test]
    fn test_roundtrip_nested() {
        let json = r#"{"outer": {"inner": [1, 2, 3], "flag": true}}"#;
        let parsed = from_str(json).unwrap();
        let serialized = serialize_value(&parsed);
        let reparsed = from_str(&serialized).unwrap();
        assert_values_equal(&parsed, &reparsed);
    }
}
