#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use std::fs;
use zparse::{
    parser::{value::Value, JsonParser, TomlParser},
    utils::{format_json, format_toml},
    Converter,
};

fn read_test_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read file: {}", path))
}

fn compare_values(left: &Value, right: &Value) -> bool {
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
                .all(|(k, v)| r_map.get(k).is_some_and(|rv| compare_values(v, rv)))
        }
        (Value::Array(l_arr), Value::Array(r_arr)) => {
            if l_arr.len() != r_arr.len() {
                return false;
            }
            l_arr
                .iter()
                .zip(r_arr.iter())
                .all(|(l, r)| compare_values(l, r))
        }
        (Value::Number(l), Value::Number(r)) => (l - r).abs() < f64::EPSILON,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

#[test]
fn test_json_to_toml_conversion() {
    let json_input = read_test_file("tests/input/file.json");
    let mut json_parser = JsonParser::new(&json_input).unwrap();
    let json_value = json_parser.parse().unwrap();

    let toml_value = Converter::json_to_toml(json_value).unwrap();

    let expected_toml = read_test_file("tests/input/file.toml");
    let mut toml_parser = TomlParser::new(&expected_toml).unwrap();
    let expected_value = toml_parser.parse().unwrap();

    assert!(
        compare_values(&toml_value, &expected_value),
        "JSON to TOML conversion produced unexpected structure.\n\
         Got:\n{}\n\nExpected:\n{}",
        format_toml(&toml_value),
        expected_toml
    );
}

#[test]
fn test_toml_to_json_conversion() {
    let toml_input = read_test_file("tests/input/file.toml");
    let mut toml_parser = TomlParser::new(&toml_input).unwrap();
    let toml_value = toml_parser.parse().unwrap();

    let json_value = Converter::toml_to_json(toml_value).unwrap();

    let expected_json = read_test_file("tests/input/file.json");
    let mut json_parser = JsonParser::new(&expected_json).unwrap();
    let expected_value = json_parser.parse().unwrap();

    assert!(
        compare_values(&json_value, &expected_value),
        "TOML to JSON conversion produced unexpected structure.\n\
         Got:\n{}\n\nExpected:\n{}",
        format_json(&json_value),
        expected_json
    );
}

#[test]
fn test_bidirectional_conversion() {
    // Test JSON -> TOML -> JSON
    let original_json = read_test_file("tests/input/file.json");
    let mut json_parser = JsonParser::new(&original_json).unwrap();
    let json_value = json_parser.parse().unwrap();

    let toml_value = Converter::json_to_toml(json_value.clone()).unwrap();
    let converted_back = Converter::toml_to_json(toml_value).unwrap();

    assert!(
        compare_values(&json_value, &converted_back),
        "JSON -> TOML -> JSON conversion did not preserve structure"
    );

    // Test TOML -> JSON -> TOML
    let original_toml = read_test_file("tests/input/file.toml");
    let mut toml_parser = TomlParser::new(&original_toml).unwrap();
    let toml_value = toml_parser.parse().unwrap();

    let json_value = Converter::toml_to_json(toml_value.clone()).unwrap();
    let converted_back = Converter::json_to_toml(json_value).unwrap();

    assert!(
        compare_values(&toml_value, &converted_back),
        "TOML -> JSON -> TOML conversion did not preserve structure"
    );
}

#[test]
fn test_specific_value_conversions() {
    let json_input = read_test_file("tests/input/file.json");
    let mut json_parser = JsonParser::new(&json_input).unwrap();
    let json_value = json_parser.parse().unwrap();

    let toml_input = read_test_file("tests/input/file.toml");
    let mut toml_parser = TomlParser::new(&toml_input).unwrap();
    let toml_value = toml_parser.parse().unwrap();

    // Test structure equality using custom comparison
    assert!(
        compare_values(&json_value, &toml_value),
        "JSON and TOML structures don't match"
    );

    // Test specific fields are present and have correct values
    if let (Value::Object(json_map), Value::Table(toml_map)) = (&json_value, &toml_value) {
        // Check success field
        assert!(compare_values(
            json_map.get("success").unwrap(),
            toml_map.get("success").unwrap()
        ));

        // Check messages array
        assert!(compare_values(
            json_map.get("messages").unwrap(),
            toml_map.get("messages").unwrap()
        ));

        // Check data array
        let json_data = json_map.get("data").unwrap();
        let toml_data = toml_map.get("data").unwrap();
        assert!(compare_values(json_data, toml_data));
    } else {
        panic!("Root value is not an object/table");
    }
}

#[test]
fn test_json_toml_json_roundtrip() {
    let json_input = read_test_file("tests/input/file.json");
    let mut json_parser = JsonParser::new(&json_input).unwrap();
    let json_value = json_parser.parse().unwrap();

    let toml_value = Converter::json_to_toml(json_value.clone()).unwrap();
    let converted_back = Converter::toml_to_json(toml_value).unwrap();

    assert!(
        compare_values(&json_value, &converted_back),
        "JSON -> TOML -> JSON conversion did not preserve structure"
    );
}

#[test]
fn test_toml_json_toml_roundtrip() {
    let toml_input = read_test_file("tests/input/file.toml");
    let mut toml_parser = TomlParser::new(&toml_input).unwrap();
    let toml_value = toml_parser.parse().unwrap();

    let json_value = Converter::toml_to_json(toml_value.clone()).unwrap();
    let converted_back = Converter::json_to_toml(json_value).unwrap();

    assert!(
        compare_values(&toml_value, &converted_back),
        "TOML -> JSON -> TOML conversion did not preserve structure"
    );
}
