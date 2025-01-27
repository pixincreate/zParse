#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

#[cfg(test)]
mod toml_tests {
    use std::{collections::HashMap, fs};
    use zparse::{
        converter::Converter,
        parser::{TomlParser, Value},
    };

    fn read_test_file(path: &str) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read file: {}", path))
    }

    // Basic Parsing Tests
    #[test]
    fn test_parse_empty_document() -> Result<(), Box<dyn std::error::Error>> {
        let input = "";
        let mut parser = TomlParser::new(input)?;
        assert_eq!(parser.parse()?, Value::Table(HashMap::new()));
        Ok(())
    }

    #[test]
    fn test_parse_basic_pairs() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
            string = "value"
            integer = 42
            float = 3.2
            boolean = true
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Table(root) = value {
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
    fn test_parse_nested_tables() -> Result<(), Box<dyn std::error::Error>> {
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

        if let Value::Table(root) = value {
            assert!(root.contains_key("server"));
            if let Some(Value::Table(server)) = root.get("server") {
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
    fn test_parse_arrays() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
            numbers = [1, 2, 3]
            strings = ["a", "b", "c"]
            mixed = [1, "two", true]
            nested = [[1, 2], [3, 4]]
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Table(root) = value {
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
    fn test_large_toml_parsing_performance() {
        let large_toml = read_test_file("tests/input/large.toml");
        let start = std::time::Instant::now();
        let mut parser = TomlParser::new(&large_toml).unwrap();
        let _ = parser.parse().unwrap();
        let duration = start.elapsed();
        println!("Time taken to parse large TOML: {:?}", duration);
        assert!(duration.as_secs() < 1, "Parsing took too long");
    }

    // Array Table Tests
    #[test]
    fn test_parse_array_tables() -> Result<(), Box<dyn std::error::Error>> {
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

        if let Value::Table(root) = value {
            if let Some(Value::Array(people)) = root.get("people") {
                assert_eq!(people.len(), 2);
                // Verify first person
                if let Value::Table(person) = &people[0] {
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
    fn test_parse_whitespace() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
            key1    =    "value1"
            key2    =    42
            [   section   ]
            key3    =    true
            "#;
        let mut parser = TomlParser::new(input)?;
        let value = parser.parse()?;

        if let Value::Table(root) = value {
            assert_eq!(root.get("key1"), Some(&Value::String("value1".to_string())));
            assert_eq!(root.get("key2"), Some(&Value::Number(42.0)));
            assert!(root.contains_key("section"));
        }
        Ok(())
    }

    // Conversion Tests
    #[test]
    fn test_toml_to_json_conversion() -> Result<(), Box<dyn std::error::Error>> {
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
        if let Value::Object(root) = json_value {
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
        let invalid_inputs = vec![
            ("[invalid", "Incomplete table header"),
            ("key = ", "Missing value"),
            ("= value", "Missing key"),
            ("[table]\nkey = 42\n[table]", "Duplicate table"),
            ("[[array]]\nkey = 1\n[array]", "Mixed array and table"),
            ("key = [1, 2, ]", "Trailing comma in array"),
            ("[a]\nb = 1\n[a.b]", "Key already defined"),
            ("[a]\nb = 1\n[[a]]", "Mixed table types"),
        ];

        for (input, error_desc) in invalid_inputs {
            let parser_result = TomlParser::new(input);
            let parse_result = match parser_result {
                Ok(mut parser) => parser.parse(),
                Err(e) => Err(e),
            };

            println!("Testing '{}': {:?}", error_desc, parse_result);
            assert!(parse_result.is_err(), "Expected error for: {}", error_desc);
        }
    }
}
