#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

#[cfg(test)]
mod toml_tests {
    use std::collections::HashMap;
    use zparse::test_utils::*;

    // Basic Parsing Tests
    #[test]
    fn test_parse_empty_document() -> Result<()> {
        let input = "";
        let mut parser = TomlParser::new(input)?;
        assert_eq!(parser.parse()?, Value::Map(HashMap::new()));
        Ok(())
    }

    #[test]
    fn test_parse_basic_pairs() -> Result<()> {
        let input = r#"
            string = "value"
            integer = 42
            float = 3.2
            boolean = true
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Map(root) = value {
            assert_eq!(
                root.get("string"),
                Some(&Value::String("value".to_string()))
            );
            assert_eq!(root.get("integer"), Some(&Value::Number(42.0)));
            assert_eq!(root.get("float"), Some(&Value::Number(3.2)));
            assert_eq!(root.get("boolean"), Some(&Value::Boolean(true)));
        } else {
            panic!("Expected table");
        }
        Ok(())
    }

    // Table Tests
    #[test]
    fn test_circular_reference_in_tables() {
        // Simple circular reference
        let input1 = r#"
            [a]
            key = "value"

            [a.b]
            key = "value"

            [a.b.a]
            key = "value"
        "#;

        let result1 = parse_toml(input1);
        assert!(
            result1.is_err(),
            "Expected error for simple circular reference"
        );
        match result1.unwrap_err().kind() {
            ParseErrorKind::Semantic(SemanticError::CircularReference) => {}
            other => panic!("Expected CircularReference error, got {:?}", other),
        }

        // More complex circular reference
        let input2 = r#"
            [x]
            key = "value"

            [x.y]
            key = "value"

            [x.y.z]
            key = "value"

            [x.y.z.x]
            key = "value"
        "#;

        let result2 = parse_toml(input2);
        assert!(
            result2.is_err(),
            "Expected error for complex circular reference"
        );
        match result2.unwrap_err().kind() {
            ParseErrorKind::Semantic(SemanticError::CircularReference) => {}
            other => panic!("Expected CircularReference error, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_table_array_access() {
        let input = r#"
            [[array]]
            key = "value"
            [array]
            other = "value"
        "#;
        let result = parse_toml(input);
        assert!(result.is_err());
        match result.unwrap_err().kind() {
            ParseErrorKind::Semantic(SemanticError::NestedTableError) => {}
            other => panic!("Expected NestedTableError error, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_nested_tables() -> Result<()> {
        let input = r#"
            [server]
            host = "localhost"
            port = 8080

            [server.ssh]
            enabled = true
            port = 22

            [server.ssh.keys]
            public = "/path/to/public"
            private = "/path/to/private"
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Map(root) = value {
            assert!(root.contains_key("server"));
            if let Some(Value::Map(server)) = root.get("server") {
                assert_eq!(
                    server.get("host"),
                    Some(&Value::String("localhost".to_string()))
                );
                assert_eq!(server.get("port"), Some(&Value::Number(8080.0)));
                assert!(server.contains_key("ssh"));
            } else {
                panic!("Invalid server table");
            }
        } else {
            panic!("Expected root table");
        }
        Ok(())
    }

    // Array Tests
    #[test]
    fn test_parse_arrays() -> Result<()> {
        let input = r#"
            numbers = [1, 2, 3]
            strings = ["a", "b", "c"]
            mixed = [1, "two", true]
            nested = [[1, 2], [3, 4]]
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Map(root) = value {
            // Test numbers array
            if let Some(Value::Array(numbers)) = root.get("numbers") {
                assert_eq!(numbers.len(), 3);
                assert_eq!(numbers[0], Value::Number(1.0));
            } else {
                panic!("Invalid numbers array");
            }

            // Test nested array
            if let Some(Value::Array(nested)) = root.get("nested") {
                assert_eq!(nested.len(), 2);
                if let Value::Array(inner) = &nested[0] {
                    assert_eq!(inner.len(), 2);
                } else {
                    panic!("Invalid nested array");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_large_toml_parsing_performance() -> Result<()> {
        let test_data = TestData::load()?;

        let start = std::time::Instant::now();
        let mut toml_parser = TomlParser::new(&test_data.large_toml)?;
        let _ = toml_parser.parse()?;
        let duration = start.elapsed();

        println!("Time taken to parse large TOML: {:?}", duration);
        assert!(duration.as_secs() < 1, "Parsing took too long");

        Ok(())
    }

    // Array Table Tests
    #[test]
    fn test_parse_array_tables() -> Result<()> {
        let input = r#"
            [[people]]
            name = "Alice"
            age = 30

            [[people]]
            name = "Bob"
            age = 25

            [[people.phones]]
            type = "home"
            number = "123-456"

            [[people.phones]]
            type = "work"
            number = "789-012"
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Map(root) = value {
            if let Some(Value::Array(people)) = root.get("people") {
                assert_eq!(people.len(), 2);
                // Verify first person
                if let Value::Map(person) = &people[0] {
                    assert_eq!(
                        person.get("name"),
                        Some(&Value::String("Alice".to_string()))
                    );
                    assert_eq!(person.get("age"), Some(&Value::Number(30.0)));
                }
            } else {
                panic!("Invalid people array");
            }
        }
        Ok(())
    }

    // Edge Cases
    #[test]
    fn test_parse_whitespace() -> Result<()> {
        let input = r#"
            key1    =    "value1"
            key2    =    42
            [   section   ]
            key3    =    true
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Map(root) = value {
            assert_eq!(root.get("key1"), Some(&Value::String("value1".to_string())));
            assert_eq!(root.get("key2"), Some(&Value::Number(42.0)));
            assert!(root.contains_key("section"));
        }
        Ok(())
    }

    // Conversion Tests
    #[test]
    fn test_toml_to_json_conversion() -> Result<()> {
        let input = r#"
            title = "Test"
            [owner]
            name = "John"
            age = 30

            [[items]]
            name = "item1"
            price = 10.99

            [[items]]
            name = "item2"
            price = 20.99
            "#;

        let mut parser = TomlParser::new(input)?;
        let toml_value = parser.parse()?;
        let json_value = Converter::toml_to_json(toml_value)?;

        // Verify the conversion maintained the structure
        if let Value::Map(root) = json_value {
            assert_eq!(root.get("title"), Some(&Value::String("Test".to_string())));
            assert!(root.contains_key("owner"));
            assert!(root.contains_key("items"));
        } else {
            panic!("Invalid conversion result");
        }
        Ok(())
    }

    // Error Cases
    #[test]
    fn test_invalid_toml() {
        let long_key = "a".repeat(1025);
        let long_key_table = format!("[{}]\nvalue = 42\n", long_key);

        let test_cases = vec![
            // Basic syntax errors
            (
                "[invalid",
                ParseErrorKind::Lexical(LexicalError::UnexpectedToken("EOF".to_string())),
            ),
            (
                "key = ",
                ParseErrorKind::Lexical(LexicalError::UnexpectedToken("EOF".to_string())),
            ),
            (
                "= value",
                ParseErrorKind::Lexical(LexicalError::UnexpectedToken("Equals".to_string())),
            ),
            // Duplicate and invalid table errors
            (
                "[table]\nkey = 42\n[table]",
                ParseErrorKind::Syntax(SyntaxError::InvalidValue(
                    "Duplicate table definition: [table]".to_string(),
                )),
            ),
            (
                "[[array]]\nkey = 1\n[array]",
                ParseErrorKind::Semantic(SemanticError::NestedTableError),
            ),
            // Array errors
            (
                "key = [1, 2, ]",
                ParseErrorKind::Syntax(SyntaxError::InvalidValue(
                    "Trailing comma in array".to_string(),
                )),
            ),
            // Nested table errors
            (
                "[a]\nb = 1\n[a.b]",
                ParseErrorKind::Semantic(SemanticError::TypeMismatch(
                    "Expected table, found Number(1)".to_string(),
                )),
            ),
            (
                "[a]\nb = 1\n[[a]]",
                ParseErrorKind::Semantic(SemanticError::NestedTableError),
            ),
            // Security limit error
            (
                long_key_table.as_str(),
                ParseErrorKind::Security(SecurityError::MaxStringLengthExceeded),
            ),
        ];

        for (input, expected_error) in test_cases {
            let parser_result = TomlParser::new(input);
            let parse_result = match parser_result {
                Ok(parser) => {
                    let config = ParserConfig {
                        max_size: 1000000,
                        max_string_length: 1024,
                        max_object_entries: 1000,
                        max_depth: 32,
                    };
                    parser.with_config(config).parse()
                }
                Err(e) => Err(e),
            };

            assert!(parse_result.is_err(), "Expected error for input: {}", input);

            let actual_error = parse_result.unwrap_err();
            println!("Testing '{}': {:?}", input, actual_error);

            // Compare error kinds more flexibly
            match (actual_error.kind(), &expected_error) {
                (ParseErrorKind::Lexical(actual), ParseErrorKind::Lexical(expected)) => {
                    assert!(
                        format!("{:?}", actual).contains(&format!("{:?}", expected)),
                        "Expected error containing '{:?}', got '{:?}' for input: {}",
                        expected,
                        actual,
                        input
                    );
                }
                (ParseErrorKind::Syntax(actual), ParseErrorKind::Syntax(expected)) => {
                    assert!(
                        format!("{:?}", actual).contains(&format!("{:?}", expected)),
                        "Expected error containing '{:?}', got '{:?}' for input: {}",
                        expected,
                        actual,
                        input
                    );
                }
                (ParseErrorKind::Semantic(actual), ParseErrorKind::Semantic(expected)) => {
                    assert!(
                        format!("{:?}", actual).contains(&format!("{:?}", expected)),
                        "Expected error containing '{:?}', got '{:?}' for input: {}",
                        expected,
                        actual,
                        input
                    );
                }
                (ParseErrorKind::Security(actual), ParseErrorKind::Security(expected)) => {
                    assert_eq!(
                        format!("{:?}", actual),
                        format!("{:?}", expected),
                        "Expected {:?}, got {:?} for input: {}",
                        expected,
                        actual,
                        input
                    );
                }
                _ => panic!(
                    "Error kind mismatch. Expected {:?}, got {:?} for input: {}",
                    expected_error,
                    actual_error.kind(),
                    input
                ),
            }
        }
    }

    #[test]
    fn toml_max_string_length_exceeded() {
        // The default max string length is 102_400. This test creates an inline table
        // with a string value that exceeds that limit.
        let long_string = "a".repeat(120_000);
        let input = format!("key = \"{}\"", long_string);
        let result = parse_toml(&input);
        assert!(
            result.is_err(),
            "Expected error for string length exceeding max"
        );

        let err = result.unwrap_err();
        match err.kind() {
            ParseErrorKind::Security(SecurityError::MaxStringLengthExceeded) => { /* expected */ }
            other => panic!(
                "Expected SecurityError::MaxStringLengthExceeded, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn toml_duplicate_table_definition() {
        // Two table headers with the same name should yield a duplicate error.
        let input = r#"
            [table]
            key = "value"

            [table]
            key2 = "value2"
        "#;
        let result = parse_toml(input);
        assert!(
            result.is_err(),
            "Expected error for duplicate table definition"
        );

        let err = result.unwrap_err();
        // The duplicate is detected as a SyntaxError with an InvalidValue message.
        match err.kind() {
            ParseErrorKind::Syntax(SyntaxError::InvalidValue(msg)) => {
                assert!(
                    msg.contains("Duplicate table definition"),
                    "Unexpected error message: {}",
                    msg
                );
            }
            other => panic!("Expected duplicate table definition error, got {:?}", other),
        }
    }
}
