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
    let toml_value = Converter::json_to_toml(json_value.clone())?;

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
    let json_value = Converter::toml_to_json(toml_value.clone())?;

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

    let toml_value = Converter::json_to_toml(json_value.clone())?;
    let converted_back = Converter::toml_to_json(toml_value.clone())?;

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

    let json_value = Converter::toml_to_json(toml_value.clone())?;
    let converted_back = Converter::json_to_toml(json_value.clone())?;

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
    let result = Converter::json_to_toml(json_value);
    assert!(result.is_err(), "Expected converter error for null value");

    let err = result.unwrap_err();
    match err.kind() {
        ParseErrorKind::Semantic(SemanticError::TypeMismatch(msg)) => {
            assert!(
                msg.contains("TOML does not support null"),
                "Unexpected error message: {}",
                msg
            );
        }
        other => panic!("Expected semantic error for null value, got {:?}", other),
    }
}
