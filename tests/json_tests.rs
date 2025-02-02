#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

#[cfg(test)]
mod json_tests {
    use std::{collections::HashMap, fs};
    use zparse::{
        converter::Converter,
        error::{LexicalError, ParseErrorKind, SyntaxError},
        parser::{config::ParserConfig, JsonParser, Value},
        utils::parse_json,
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
            // Add new error cases
            ("\"invalid\\uXYZZ\"", "Invalid Unicode escape"),
            ("\"bad\\escape\"", "Invalid escape sequence"),
            ("12e999", "Number overflow"),
            ("-12e999", "Number underflow"),
            ("{\"key\":}", "Invalid value"),
            ("{\"key\" \"value\"}", "Missing colon"),
            ("{\"key\":1,}", "Trailing comma"),
        ];

        for (input, error_desc) in invalid_inputs {
            let parser_result = JsonParser::new(input);
            let parse_result = match parser_result {
                Ok(parser) => {
                    // Test with custom config for security validations
                    let config = ParserConfig {
                        max_size: 100,
                        max_string_length: 20,
                        max_object_entries: 5,
                        max_depth: 3,
                    };
                    parser.with_config(config).parse()
                }
                Err(e) => Err(e),
            };

            assert!(parse_result.is_err(), "Expected error for: {}", error_desc);

            // Verify specific error types
            match parse_result.unwrap_err().kind() {
                ParseErrorKind::Lexical(_) => {}
                ParseErrorKind::Syntax(_) => {}
                ParseErrorKind::Security(_) => {}
                _ => panic!("Unexpected error type for: {}", error_desc),
            }
        }
    }

    #[test]
    fn json_missing_comma() {
        // This JSON is missing a comma between key/value pairs.
        let input = r#"{"key1": "value1" "key2": "value2"}"#;
        let result = parse_json(input);
        assert!(result.is_err(), "Expected error for missing comma");

        let err = result.unwrap_err();
        match err.kind() {
            ParseErrorKind::Syntax(SyntaxError::MissingComma) => { /* expected */ }
            other => panic!("Expected missing comma error, got {:?}", other),
        }
    }

    #[test]
    fn json_trailing_comma() {
        // A trailing comma in the JSON object should result in an error.
        let input = r#"{"key1": "value1", "key2": "value2",}"#;
        let result = parse_json(input);
        assert!(result.is_err(), "Expected error for trailing comma");

        let err = result.unwrap_err();
        // In our JSON parser, a trailing comma causes an unexpected token error.
        match err.kind() {
            ParseErrorKind::Lexical(LexicalError::UnexpectedToken(msg)) => {
                assert!(
                    msg.contains("Trailing"),
                    "Error message should mention a trailing comma, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected LexicalError::UnexpectedToken for trailing comma, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn json_invalid_unquoted_value() {
        // In JSON mode, a bare (unquoted) value is not allowed.
        let input = r#"{"key": "value", "another_key": value}"#;
        let result = parse_json(input);
        assert!(
            result.is_err(),
            "Expected error for unquoted string for value"
        );

        let err = result.unwrap_err();
        match err.kind() {
            ParseErrorKind::Lexical(LexicalError::InvalidToken(msg)) => {
                assert!(
                    msg.contains("Unexpected char"),
                    "Error message should indicate an unexpected character, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected LexicalError::InvalidToken due to unquoted value, got {:?}",
                other
            ),
        }
    }
}
