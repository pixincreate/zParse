#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use std::fs;

use zparse::test_utils::*;

#[test]
fn file_read_error() {
    // Attempt reading a non-existent file should produce an error.
    let non_existent = "nonexistent_file.json";
    let result = parse_file(non_existent);
    assert!(
        result.is_err(),
        "Expected error when reading non-existent file"
    );

    let err = result.unwrap_err();
    match err.kind() {
        ParseErrorKind::IO(_) => { /* expected */ }
        other => panic!("Expected IO error, got {:?}", other),
    }
}

#[test]
fn read_and_write_file() {
    // Use a unique file name for this test.
    let temp_path = tmp_file_path("rw_test.txt");
    let temp_path_str = temp_path.to_str().expect("valid path");

    let content = "Hello, zParse!";
    // Write file using write_file utility.
    write_file(temp_path_str, content).expect("Failed to write file");

    // Read back file using read_file utility.
    let read_content = read_file(temp_path_str).expect("Failed to read file");
    assert_eq!(content, read_content);

    // Clean up the temporary file.
    let _ = fs::remove_file(temp_path);
}

#[test]
fn parse_and_format_json_file() {
    // Create a temporary JSON file.
    let temp_path = tmp_file_path("test.json");
    let temp_path_str = temp_path.to_str().expect("valid path");

    // A simple JSON string.
    let json_content = r#"{
         "key": "value",
         "array": [1, 2, 3]
     }"#;

    fs::write(temp_path_str, json_content).expect("Failed to write JSON file");

    // Use parse_file to parse the JSON file.
    let parsed = parse_file(temp_path_str).expect("Failed to parse JSON file");
    // Format back using the JSON formatter.
    let formatted = format_json(&parsed);
    // Check that the formatted output contains at least one expected element.
    assert!(
        formatted.contains("\"key\""),
        "Formatted output should contain key"
    );
    assert!(
        formatted.contains("value"),
        "Formatted output should contain value"
    );

    // Clean up the temporary file.
    let _ = fs::remove_file(temp_path);
}

#[test]
fn parse_and_format_toml_file() {
    // Create a temporary TOML file.
    let temp_path = tmp_file_path("test.toml");
    let temp_path_str = temp_path.to_str().expect("valid path");

    // A simple TOML string.
    let toml_content = r#"
 key = "value"
 array = [1, 2, 3]
 "#;
    fs::write(temp_path_str, toml_content).expect("Failed to write TOML file");

    // Use parse_file to parse the TOML file.
    let parsed = parse_file(temp_path_str).expect("Failed to parse TOML file");
    // Format using the TOML formatter.
    let formatted = format_toml(&parsed);
    // Check that the formatted output contains expected key names.
    assert!(
        formatted.contains("key"),
        "Formatted output should contain key"
    );
    assert!(
        formatted.contains("value"),
        "Formatted output should contain value"
    );

    // Clean up the temporary file.
    let _ = fs::remove_file(temp_path);
}

#[test]
fn unknown_file_extension() {
    // Use a unique file name for this test.
    let temp_path = tmp_file_path("unknown_test.txt");
    let temp_path_str = temp_path.to_str().expect("valid path");

    fs::write(temp_path_str, "some content").expect("Failed to write file");

    let result = parse_file(temp_path_str);
    assert!(
        result.is_err(),
        "Parsing a file with an unknown extension should error"
    );
    if let Err(err) = result {
        match err.kind() {
            // In our main parser, unknown extensions trigger a Semantic error with UnknownFormat.
            ParseErrorKind::Semantic(SemanticError::UnknownFormat) => (),
            other => panic!("Expected unknown format error, got {:?}", other),
        }
    }

    let _ = fs::remove_file(temp_path);
}
