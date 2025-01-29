#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use zparse::error::{ParseErrorKind, SecurityError};
use zparse::parser::config::{DEFAULT_MAX_DEPTH, DEFAULT_MAX_OBJECT_ENTRIES, DEFAULT_MAX_SIZE};
use zparse::parser::{JsonParser, TomlParser};

#[test]
fn test_max_input_size() {
    // Create input exactly larger than max size (1MB)
    let size = DEFAULT_MAX_SIZE + 1;
    let large_input = "0".repeat(size);

    let result = JsonParser::new(&large_input);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e.kind(),
            ParseErrorKind::Security(SecurityError::MaxSizeExceeded)
        ));
    }
}

#[test]
fn test_max_object_entries() {
    // Create object with more than max entries (1000)
    let mut entries = Vec::with_capacity(DEFAULT_MAX_OBJECT_ENTRIES + 1);
    for i in 0..=DEFAULT_MAX_OBJECT_ENTRIES {
        entries.push(format!(r#""key{}":0"#, i));
    }
    let json = format!("{{{}}}", entries.join(","));

    let result = JsonParser::new(&json).and_then(|mut parser| parser.parse());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e.kind(),
            ParseErrorKind::Security(SecurityError::MaxObjectEntriesExceeded)
        ));
    }
}

#[test]
fn test_stack_overflow_prevention() {
    // Create deeply nested TOML structure that exceeds max_depth
    let mut toml = String::new();
    let mut tables = Vec::new();

    // Create one more table than the max depth allows
    for i in 0..=DEFAULT_MAX_DEPTH {
        tables.push(format!("table{}", i));
        let table_path = tables.join(".");
        toml.push_str(&format!("[{}]\n", table_path));
    }

    let result = TomlParser::new(&toml).and_then(|mut parser| parser.parse());
    assert!(
        result.is_err(),
        "Expected error for excessive nesting, got {:?}",
        result
    );
    if let Err(e) = result {
        assert!(
            matches!(
                e.kind(),
                ParseErrorKind::Security(SecurityError::MaxDepthExceeded)
            ),
            "Expected MaxDepthExceeded, got {:?}",
            e.kind()
        );
    }
}

#[test]
fn test_max_string_length() {
    // Create string larger than max length (100KB)
    let long_string = "x".repeat(101 * 1024);
    let json = format!(r#""{}""#, long_string);

    let result = JsonParser::new(&json).and_then(|mut parser| parser.parse());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e.kind(),
            ParseErrorKind::Security(SecurityError::MaxStringLengthExceeded)
        ));
    }
}

#[test]
fn test_max_nesting_depth() {
    // Create deeply nested JSON structure
    let mut json = String::new();
    for _ in 0..33 {
        // Just over max_depth of 32
        json.push('{');
        json.push_str(r#""x":"#);
    }
    json.push_str(r#""value""#);
    json.push_str(&"}".repeat(33));

    let result = JsonParser::new(&json).and_then(|mut parser| parser.parse());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e.kind(),
            ParseErrorKind::Security(SecurityError::MaxDepthExceeded)
        ));
    }
}
