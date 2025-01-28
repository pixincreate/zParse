#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::as_conversions)]

use proptest::collection::vec;
use proptest::prelude::*;
use zparse::{parser::JsonParser, Converter, Value};

// Helper function to compare values structurally rather than string representation
fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Map(l_map), Value::Map(r_map))=> {
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

// Strategy for generating valid JSON strings
fn json_string_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-\\.\\s]{1,50}".prop_map(|s| s.replace('\\', "\\\\"))
}

// Strategy for generating arrays of numbers
fn number_array_strategy() -> impl Strategy<Value = Vec<f64>> {
    vec(-1000.0..1000.0f64, 0..10)
}

proptest! {
    // Basic Tests
    #[test]
    fn test_basic_json_roundtrip(s in json_string_strategy()) {
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


    // Nested Structure Tests
    #[test]
    fn test_nested_objects(
        key1 in json_string_strategy(),
        key2 in json_string_strategy(),
        n in -1000i32..1000i32
    ) {
        let json_str = format!(
            r#"{{
                "outer": {{
                    "inner1": {{
                        "key1": "{}",
                        "value": {}
                    }},
                    "inner2": {{
                        "key2": "{}"
                    }}
                }}
            }}"#,
            key1, n, key2
        );

        let mut parser = JsonParser::new(&json_str).unwrap();
        let original = parser.parse().unwrap();
        let toml = Converter::json_to_toml(original.clone()).unwrap();
        let back_to_json = Converter::toml_to_json(toml).unwrap();

        prop_assert!(values_equal(&original, &back_to_json));
    }

    // Array Tests
    #[test]
    fn test_complex_arrays(
            numbers in number_array_strategy(),
            strings in vec(json_string_strategy(), 0..5)
        ) {
            let numbers_str = numbers.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let strings_str = strings.iter()
                .map(|s| format!(r#""{}""#, s))
                .collect::<Vec<_>>()
                .join(",");

            let json_str = format!(
                r#"{{
                    "numbers": [{}],
                    "strings": [{}],
                    "mixed": [1, "test", true],
                    "nested": [[1,2], [3,4], [5,6]]
                }}"#,
                numbers_str, strings_str
            );

            let mut parser = JsonParser::new(&json_str).unwrap();
            let original = parser.parse().unwrap();
            let toml = Converter::json_to_toml(original.clone()).unwrap();
            let back_to_json = Converter::toml_to_json(toml).unwrap();

            prop_assert!(values_equal(&original, &back_to_json));
        }

    // Edge Cases
    #[test]
    fn test_edge_cases(s in json_string_strategy()) {
        let edge_cases = vec![
            // Objects and tables
            r#"{"empty_object":{}}"#.to_string(),
            r#"{"nested_empty":{"inner":{}}}"#.to_string(),

            // Numbers
            r#"{"integer":42}"#.to_string(),
            r#"{"float":3.14}"#.to_string(),
            r#"{"negative":-123}"#.to_string(),

            // Strings
            format!(r#"{{"string":"{}"}}"#, s),
            r#"{"escaped":"\"\n\r\t"}"#.to_string(),

            // Arrays
            r#"{"array":[1,2,3]}"#.to_string(),
            r#"{"nested_array":[[1],[2],[3]]}"#.to_string(),

            // Mixed valid types
            format!(r#"{{"mixed":{{"num":42,"str":"{}","bool":true}}}}"#, s),
        ];

        for case in edge_cases {
            let mut parser = JsonParser::new(&case).unwrap();
            let original = parser.parse().unwrap();
            let toml = Converter::json_to_toml(original.clone()).unwrap();
            let back_to_json = Converter::toml_to_json(toml).unwrap();

            prop_assert!(values_equal(&original, &back_to_json));
        }
    }

    // Special Number Tests
    #[test]
    fn test_number_formats(n in -1000000.0..1000000.0f64) {
            let json_str = format!(
                r#"{{
                    "regular": {},
                    "fixed": {:.6},
                    "integer": {},
                    "array": [{}, {:.2}]
                }}"#,
                n, n, n as i64, n, n
            );

            let mut parser = JsonParser::new(&json_str).unwrap();
            let original = parser.parse().unwrap();
            let toml = Converter::json_to_toml(original.clone()).unwrap();
            let back_to_json = Converter::toml_to_json(toml).unwrap();

            prop_assert!(values_equal(&original, &back_to_json));
        }

    // Deep Structure Tests
    #[test]
    fn test_deep_structure(
        s1 in json_string_strategy(),
        s2 in json_string_strategy(),
        n in -100..100i32,
        b in any::<bool>()
    ) {
        let json_str = format!(
            r#"{{
                "level1": {{
                    "level2": {{
                        "level3": {{
                            "level4": {{
                                "string1": "{}",
                                "string2": "{}",
                                "number": {},
                                "boolean": {},
                                "array": [
                                    {{"id": 1}},
                                    {{"id": 2}},
                                    {{"nested": {{"deep": "value"}}}}
                                ]
                            }}
                        }}
                    }}
                }}
            }}"#,
            s1, s2, n, b
        );

        let mut parser = JsonParser::new(&json_str).unwrap();
        let original = parser.parse().unwrap();
        let toml = Converter::json_to_toml(original.clone()).unwrap();
        let back_to_json = Converter::toml_to_json(toml).unwrap();

        prop_assert!(values_equal(&original, &back_to_json));
    }

    // Array Table Tests (TOML-specific)
    #[test]
    fn test_array_tables(
        key in json_string_strategy(),
        values in vec(json_string_strategy(), 1..5)
    ) {
        let items: Vec<String> = values.iter()
            .enumerate()
            .map(|(i, v)| format!(
                r#"{{"id":{},"key":"{}","value":"{}"}}"#,
                i, key, v
            ))
            .collect();

        let json_str = format!(
            r#"{{
                "items": [{}]
            }}"#,
            items.join(",")
        );

        let mut parser = JsonParser::new(&json_str).unwrap();
        let original = parser.parse().unwrap();
        let toml = Converter::json_to_toml(original.clone()).unwrap();
        let back_to_json = Converter::toml_to_json(toml).unwrap();

        prop_assert!(values_equal(&original, &back_to_json));
    }

    // Mixed Complex Types
    #[test]
    fn test_mixed_complex_types(
            s in json_string_strategy(),
            numbers in number_array_strategy(),
            b in any::<bool>()
        ) {
            let numbers_str = numbers.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let json_str = format!(
                r#"{{
                    "metadata": {{
                        "name": "{}",
                        "enabled": {},
                        "counts": [{}]
                    }},
                    "data": {{
                        "simple": "value",
                        "array": [1, 2, 3],
                        "object": {{"nested": "value"}},
                        "mixed": [
                            {{"type": "object"}},
                            [1, 2, 3],
                            {{"another": "object"}}
                        ]
                    }}
                }}"#,
                s, b, numbers_str
            );

            let mut parser = JsonParser::new(&json_str).unwrap();
            let original = parser.parse().unwrap();
            let toml = Converter::json_to_toml(original.clone()).unwrap();
            let back_to_json = Converter::toml_to_json(toml).unwrap();

            prop_assert!(values_equal(&original, &back_to_json));
        }
}
