use zparse::{convert, Format};

#[test]
fn test_json_to_toml() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"name":"test","value":42}"#;
    let output = convert(input, Format::Json, Format::Toml)?;
    assert!(output.contains("name"));
    assert!(output.contains("value"));
    Ok(())
}

#[test]
fn test_toml_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "name = \"test\"\nvalue = 42\n";
    let output = convert(input, Format::Toml, Format::Json)?;
    assert!(output.contains("\"name\""));
    assert!(output.contains("\"value\""));
    Ok(())
}

#[test]
fn test_yaml_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "name: test\nvalue: 42\n";
    let output = convert(input, Format::Yaml, Format::Json)?;
    assert!(output.contains("\"name\""));
    assert!(output.contains("\"value\""));
    Ok(())
}

#[test]
fn test_xml_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "<root><name>test</name><value>42</value></root>";
    let output = convert(input, Format::Xml, Format::Json)?;
    assert!(output.contains("\"root\""));
    assert!(output.contains("\"name\""));
    Ok(())
}
