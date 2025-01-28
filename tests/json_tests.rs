#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

#[cfg(test)]
mod json_tests {
    use std::{collections::HashMap, fs};
    use zparse::{
        converter::Converter,
        parser::{JsonParser, Value},
    };

    fn read_test_file(path: &str) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read file: {}", path))
    }

    // Basic Parsing Tests
    #[test]
    fn test_parse_empty_object() -> Result<(), Box<dyn std::error::Error>> {
        let input = "{}";
        let mut parser = JsonParser::new(input)?;
        assert_eq!(parser.parse()?, Value::Map(HashMap::new()));
        Ok(())
    }

    #[test]
    fn test_parse_empty_array() -> Result<(), Box<dyn std::error::Error>> {
        let input = "[]";
        let mut parser = JsonParser::new(input)?;
        assert_eq!(parser.parse()?, Value::Array(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_primitive_values() -> Result<(), Box<dyn std::error::Error>> {
        let inputs = vec![
            ("42", Value::Number(42.0)),
            ("-42.5", Value::Number(-42.5)),
            ("true", Value::Boolean(true)),
            ("false", Value::Boolean(false)),
            ("null", Value::Null),
            ("\"hello\"", Value::String("hello".to_string())),
        ];

        for (input, expected) in inputs {
            let mut parser = JsonParser::new(input)?;
            assert_eq!(parser.parse()?, expected);
        }
        Ok(())
    }

    #[test]
    fn test_large_json_parsing_performance() {
        let large_json = read_test_file("tests/input/large.json");
        let start = std::time::Instant::now();
        let mut parser = JsonParser::new(&large_json).unwrap();
        let _ = parser.parse().unwrap();
        let duration = start.elapsed();
        println!("Time taken to parse large JSON: {:?}", duration);
        assert!(duration.as_secs() < 1, "Parsing took too long");
    }

    // Object Tests
    #[test]
    fn test_parse_simple_object() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"{"name": "John", "age": 30, "is_student": false}"#;
        let mut parser = JsonParser::new(input)?;
        let value = parser.parse()?;

        let mut expected = HashMap::new();
        expected.insert("name".to_string(), Value::String("John".to_string()));
        expected.insert("age".to_string(), Value::Number(30.0));
        expected.insert("is_student".to_string(), Value::Boolean(false));

        assert_eq!(value, Value::Map(expected));
        Ok(())
    }

    #[test]
    fn test_parse_nested_objects() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
        {
            "person": {
                "name": {
                    "first": "John",
                    "last": "Doe"
                },
                "contact": {
                    "email": "john@example.com",
                    "phone": {
                        "home": "123-456",
                        "work": "789-012"
                    }
                }
            }
        }"#;
        let mut parser = JsonParser::new(input)?;
        let value = parser.parse()?;

        // Verify structure exists
        if let Value::Map(root) = value {
            if let Some(Value::Map(person)) = root.get("person") {
                assert!(person.contains_key("name"));
                assert!(person.contains_key("contact"));
            } else {
                panic!("Invalid person object");
            }
        } else {
            panic!("Invalid root object");
        }
        Ok(())
    }

    // Array Tests
    #[test]
    fn test_parse_arrays() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"[1, "two", true, null, [2.5, false], {"key": "value"}]"#;
        let mut parser = JsonParser::new(input)?;
        let value = parser.parse()?;

        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));

        let expected = Value::Array(vec![
            Value::Number(1.0),
            Value::String("two".to_string()),
            Value::Boolean(true),
            Value::Null,
            Value::Array(vec![Value::Number(2.5), Value::Boolean(false)]),
            Value::Map(obj),
        ]);

        assert_eq!(value, expected);
        Ok(())
    }

    // Edge Cases
    #[test]
    fn test_parse_whitespace() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
        {
            "key1"     :    "value1"    ,
            "key2"     :    42
        }
        "#;
        let mut parser = JsonParser::new(input)?;
        let value = parser.parse()?;

        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), Value::String("value1".to_string()));
        expected.insert("key2".to_string(), Value::Number(42.0));

        assert_eq!(value, Value::Map(expected));
        Ok(())
    }

    #[test]
    fn test_parse_escaped_strings() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"{"text": "Hello\nWorld\t\"Escaped\""}"#;
        let mut parser = JsonParser::new(input)?;
        let value = parser.parse()?;

        let mut expected = HashMap::new();
        expected.insert(
            "text".to_string(),
            Value::String("Hello\nWorld\t\"Escaped\"".to_string()),
        );

        assert_eq!(value, Value::Map(expected));
        Ok(())
    }

    // Conversion Tests
    #[test]
    fn test_json_to_toml_conversion() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"{
            "title": "Test",
            "owner": {
                "name": "John",
                "age": 30
            },
            "database": {
                "ports": [8000, 8001, 8002],
                "enabled": true
            }
        }"#;

        let mut parser = JsonParser::new(input)?;
        let json_value = parser.parse()?;
        let toml_value = Converter::json_to_toml(json_value)?;

        // Verify the conversion maintained the structure
        if let Value::Map(root) = toml_value {
            assert_eq!(root.get("title"), Some(&Value::String("Test".to_string())));
            assert!(root.contains_key("owner"));
            assert!(root.contains_key("database"));
        } else {
            panic!("Invalid conversion result");
        }
        Ok(())
    }

    // Error Cases
    #[test]
    fn test_invalid_json() {
        let invalid_inputs = vec![
            ("{", "Incomplete object"),
            ("[", "Incomplete array"),
            ("}", "Unexpected closing brace"),
            ("]", "Unexpected closing bracket"),
            ("{\"key\"}", "Missing value"),
            ("{key: \"value\"}", "Unquoted key"),
            ("[1, 2,]", "Trailing comma"),
            ("\"unclosed string", "Unclosed string"),
        ];

        for (input, error_desc) in invalid_inputs {
            // Create parser and check both creation and parsing
            let parser_result = JsonParser::new(input);
            let parse_result = match parser_result {
                Ok(mut parser) => parser.parse(),
                Err(e) => Err(e),
            };

            assert!(parse_result.is_err(), "Expected error for: {}", error_desc);
        }
    }
}
