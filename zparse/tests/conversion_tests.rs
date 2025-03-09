#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use zparse::test_utils::*;

#[test]
fn test_json_to_toml_conversion() -> Result<()> {
    let test_data = TestData::load()?;

    let mut json_parser = JsonParser::new(&test_data.medium_json)?;
    let json_value = json_parser.parse()?;
    let toml_value = Converter::json_to_toml(&json_value)?;

    let mut toml_parser = TomlParser::new(&test_data.medium_toml)?;
    let expected_value = toml_parser.parse()?;

    assert_values_equal(
        &toml_value,
        &expected_value,
        "Values don't match after conversion",
    );
    Ok(())
}

#[test]
fn test_toml_to_json_conversion() -> Result<()> {
    let test_data = TestData::load()?;

    let mut toml_parser = TomlParser::new(&test_data.medium_toml)?;
    let toml_value = toml_parser.parse()?;
    let json_value = Converter::toml_to_json(&toml_value)?;

    let mut json_parser = JsonParser::new(&test_data.medium_json)?;
    let expected_value = json_parser.parse()?;

    assert_values_equal(
        &json_value,
        &expected_value,
        "Values don't match after conversion",
    );
    Ok(())
}

#[test]
fn test_bidirectional_conversion() -> Result<()> {
    test_json_toml_json_roundtrip()?;
    test_toml_json_toml_roundtrip()?;
    Ok(())
}

#[test]
fn test_specific_value_conversions() -> Result<()> {
    let test_data = TestData::load()?;

    let mut json_parser = JsonParser::new(&test_data.medium_json)?;
    let json_value = json_parser.parse()?;

    let mut toml_parser = TomlParser::new(&test_data.medium_toml)?;
    let toml_value = toml_parser.parse()?;

    assert_values_equal(
        &json_value,
        &toml_value,
        "JSON and TOML structures don't match",
    );

    // Test specific fields are present and have correct values
    if let (Value::Map(json_map), Value::Map(toml_map)) = (&json_value, &toml_value) {
        assert_values_equal(
            json_map.get("success").unwrap(),
            toml_map.get("success").unwrap(),
            "Success fields don't match",
        );

        assert_values_equal(
            json_map.get("messages").unwrap(),
            toml_map.get("messages").unwrap(),
            "Messages arrays don't match",
        );

        assert_values_equal(
            json_map.get("data").unwrap(),
            toml_map.get("data").unwrap(),
            "Data arrays don't match",
        );
    } else {
        panic!("Root value is not an object/table");
    }

    Ok(())
}

fn test_json_toml_json_roundtrip() -> Result<()> {
    // Test JSON -> TOML -> JSON
    let test_data = TestData::load()?;

    let mut original_json = JsonParser::new(&test_data.medium_json)?;
    let json_value = original_json.parse()?;

    let toml_value = Converter::json_to_toml(&json_value)?;
    let converted_back = Converter::toml_to_json(&toml_value)?;

    assert_values_equal(
        &toml_value,
        &converted_back,
        "JSON -> TOML -> JSON conversion did not preserve structure",
    );

    Ok(())
}

fn test_toml_json_toml_roundtrip() -> Result<()> {
    // Test TOML -> JSON -> TOML
    let test_data = TestData::load()?;

    let mut original_toml = TomlParser::new(&test_data.medium_toml)?;
    let toml_value = original_toml.parse()?;

    let json_value = Converter::toml_to_json(&toml_value)?;
    let converted_back = Converter::json_to_toml(&json_value)?;

    assert_values_equal(
        &json_value,
        &converted_back,
        "TOML -> JSON -> TOML conversion did not preserve structure",
    );

    Ok(())
}

#[test]
fn converter_null_value_error() {
    // Converting a JSON object containing a null value to TOML should fail
    // because TOML does not support null values.
    let input = r#"{"key": null}"#;
    let json_value = parse_json(input).expect("JSON should parse successfully");
    let result = Converter::json_to_toml(&json_value);
    assert!(result.is_err(), "Expected converter error for null value");

    let err = result.unwrap_err();
    match err.kind() {
        // Update this match arm to expect ConversionError instead of SemanticError
        ParseErrorKind::Conversion(ConversionError::UnsupportedValue(msg)) => {
            assert!(
                msg.contains("Null value"),
                "Unexpected error message: {}",
                msg
            );
        }
        other => panic!("Expected conversion error for null value, got {:?}", other),
    }
}

#[test]
fn test_array_with_null_conversion() {
    let input = r#"{"array": [1, null, 3]}"#;
    let json_value = parse_json(input).expect("JSON should parse successfully");
    let result = Converter::json_to_toml(&json_value);
    assert!(result.is_err(), "Expected error for array containing null");

    let err = result.unwrap_err();
    match err.kind() {
        ParseErrorKind::Conversion(ConversionError::UnsupportedValue(msg)) => {
            assert!(
                msg.contains("Null values in arrays"),
                "Unexpected error message: {}",
                msg
            );
        }
        other => panic!(
            "Expected conversion error for array with null, got {:?}",
            other
        ),
    }
}

#[test]
fn test_error_location_tracking() -> Result<()> {
    let input = r#"{"key": null}"#; // Invalid TOML value
    let json_value = parse_json(input)?;
    let result = Converter::json_to_toml(&json_value);

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Verify location info
    if let Some(loc) = err.location() {
        assert_eq!(loc.line, 1, "Error should be on first line");
        assert_eq!(loc.column, 6, "Error should point to null value position");

        match err.kind() {
            ParseErrorKind::Conversion(ConversionError::UnsupportedValue(msg)) => {
                assert!(msg.contains("Null value"), "Expected null value error");
            }
            other => panic!(
                "Expected ConversionError::UnsupportedValue, got {:?}",
                other
            ),
        }
    } else {
        panic!("Missing location info");
    }
    Ok(())
}

#[test]
fn test_nested_location_tracking() -> Result<()> {
    let input = r#"{
        "outer": {
            "inner": null
        }
    }"#;
    let json_value = parse_json(input)?;
    let result = Converter::json_to_toml(&json_value);

    assert!(result.is_err());
    let err = result.unwrap_err();

    if let Some(loc) = err.location() {
        // Check that we have reasonable location values
        assert!(loc.line > 0, "Should have a line number");
        assert!(loc.column > 0, "Should have a column number");

        // Verify error context contains path information
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("outer.inner"),
            "Error should contain path information: {}",
            err_msg
        );

        // Verify error kind
        match err.kind() {
            ParseErrorKind::Conversion(ConversionError::UnsupportedValue(msg)) => {
                assert!(
                    msg.contains("Null value"),
                    "Error message should mention null value"
                );
            }
            other => panic!(
                "Expected ConversionError::UnsupportedValue, got {:?}",
                other
            ),
        }
    } else {
        panic!("Missing location info");
    }
    Ok(())
}
