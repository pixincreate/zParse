#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use proptest::prelude::*;
use zparse::{parser::JsonParser, Converter, Value};

// Helper function to compare values structurally rather than string representation
fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Object(l_map), Value::Object(r_map))
        | (Value::Table(l_map), Value::Table(r_map))
        | (Value::Object(l_map), Value::Table(r_map))
        | (Value::Table(l_map), Value::Object(r_map)) => {
            if l_map.len() != r_map.len() {
                return false;
            }
            l_map
                .iter()
                .all(|(k, v)| r_map.get(k).is_some_and(|r_v| values_equal(v, r_v)))
        }
        (Value::Array(l_arr), Value::Array(r_arr)) => {
            if l_arr.len() != r_arr.len() {
                return false;
            }
            l_arr
                .iter()
                .zip(r_arr.iter())
                .all(|(l, r)| values_equal(l, r))
        }
        (Value::Number(l), Value::Number(r)) => (l - r).abs() < f64::EPSILON,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Null, Value::Null) => true,
        (Value::DateTime(l), Value::DateTime(r)) => l == r,
        _ => false,
    }
}

proptest! {
    #[test]
    fn json_roundtrip(s in "[a-zA-Z0-9]*") {
        let json_str = format!(
            r#"{{"string":"{}","number":42.5,"boolean":true,"array":[1,2,3]}}"#,
            s
        );

        let mut parser = JsonParser::new(&json_str).unwrap();
        let parsed = parser.parse().unwrap();

        let mut parser = JsonParser::new(&parsed.to_string()).unwrap();
        let reparsed = parser.parse().unwrap();

        prop_assert!(values_equal(&parsed, &reparsed));
    }

    #[test]
    fn conversion_roundtrip(s in "[a-zA-Z0-9]*") {
        let json_str = format!(
            r#"{{"key":"{}","nested":{{"array":[1,2,3],"bool":true}}}}"#,
            s
        );

        let mut parser = JsonParser::new(&json_str).unwrap();
        let original = parser.parse().unwrap();

        // JSON -> TOML -> JSON roundtrip
        let toml = Converter::json_to_toml(original.clone()).unwrap();
        let back_to_json = Converter::toml_to_json(toml).unwrap();

        prop_assert!(values_equal(&original, &back_to_json));
    }
}
