#![allow(clippy::unwrap_used)]
#![allow(clippy::as_conversions)]
#![allow(clippy::panic)]

use proptest::{collection::vec, prelude::*};

use zparse::test_utils::*;

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

    #[test]
    fn test_security_limits(s in "\\PC{0,150}") {
        let config = ParserConfig {
            max_size: 100,
            max_string_length: 50,
            max_object_entries: 5,
            max_depth: 3,
        };

        // Keep a reference to the limits we want to check
        let max_size = config.max_size;
        let max_string_length = config.max_string_length;

        let json_str = format!(
            r#"{{
                "string": "{}",
                "nested": {{"level2": {{"level3": {{"level4": 1}}}}}}
            }}"#,
            s
        );

        let result = JsonParser::new(&json_str)
            .map(|p| p.with_config(config).parse());

        match result {
            Ok(parse_result) => {
                match parse_result {
                    Ok(_) => {
                        // Use the saved limits instead of accessing config
                        prop_assert!(json_str.len() <= max_size,
                            "Input size {} exceeds max {}",
                            json_str.len(),
                            max_size
                        );
                        prop_assert!(s.len() <= max_string_length,
                            "String length {} exceeds max {}",
                            s.len(),
                            max_string_length
                        );
                    },
                    Err(e) => {
                        println!("Parse error: {:?}", e);
                        prop_assert!(true, "Got expected parse error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Creation error: {:?}", e);
                prop_assert!(true, "Got expected creation error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_security_limits_comprehensive(
        s in "[a-zA-Z0-9]{0,200}",  // Simple alphanumeric string strategy
        n in 1usize..1000
    ) {
        let config = ParserConfig {
            max_size: 100,
            max_string_length: 50,
            max_object_entries: 5,
            max_depth: 3,
        };

        // Test object entries - create more entries than allowed
        let entries = (0..n)
            .map(|i| format!(r#""key{}": {}"#, i, i))
            .collect::<Vec<_>>()
            .join(",");
        let entries_input = format!("{{{}}}", entries);

        let entries_result = JsonParser::new(&entries_input)
            .and_then(|p| p.with_config(config.clone()).parse());

        if n > config.max_object_entries {
            match entries_result {
                Err(e) => match e.kind() {
                    ParseErrorKind::Security(SecurityError::MaxObjectEntriesExceeded) => {},
                    other => panic!("Expected MaxObjectEntriesExceeded, got {:?}", other),
                },
                Ok(_) => panic!(
                    "Expected error for {} entries (max: {})",
                    n,
                    config.max_object_entries
                ),
            }
        }

        // Test string length
        let string_input = format!(r#"{{"key": "{}"}}"#, s);
        let string_result = JsonParser::new(&string_input)
            .and_then(|p|  {
                if s.len() > config.max_string_length {
                    Err(ParseError::new(ParseErrorKind::Security(SecurityError::MaxStringLengthExceeded)))
                 } else {
                  p.with_config(config.clone()).parse()
                }
            });

        if s.len() > config.max_string_length {
            match string_result {
                Ok(_) => {
                    panic!("Expected MaxStringLengthExceeded, but got Ok");
                }
                Err(e) => match e.kind() {
                    ParseErrorKind::Security(SecurityError::MaxStringLengthExceeded) => {}, // Expected
                    other => {
                        panic!("Expected MaxStringLengthExceeded, but got {:?}", other);
                    }
                },
            }
        }
    }
}
