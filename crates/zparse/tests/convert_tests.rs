use zparse::{Format, convert};

#[test]
fn test_json_to_toml() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"name":"test","value":42}"#;
    let output = convert(input, Format::Json, Format::Toml)?;
    if !output.contains("name") {
        return Err("missing name in toml output".into());
    }
    if !output.contains("value") {
        return Err("missing value in toml output".into());
    }
    Ok(())
}

#[test]
fn test_toml_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "name = \"test\"\nvalue = 42\n";
    let output = convert(input, Format::Toml, Format::Json)?;
    if !output.contains("\"name\"") {
        return Err("missing name in json output".into());
    }
    if !output.contains("\"value\"") {
        return Err("missing value in json output".into());
    }
    Ok(())
}

#[test]
fn test_yaml_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "name: test\nvalue: 42\n";
    let output = convert(input, Format::Yaml, Format::Json)?;
    if !output.contains("\"name\"") {
        return Err("missing name in json output".into());
    }
    if !output.contains("\"value\"") {
        return Err("missing value in json output".into());
    }
    Ok(())
}

#[test]
fn test_xml_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "<root><name>test</name><value>42</value></root>";
    let output = convert(input, Format::Xml, Format::Json)?;
    if !output.contains("\"root\"") {
        return Err("missing root in json output".into());
    }
    if !output.contains("\"name\"") {
        return Err("missing name in json output".into());
    }
    Ok(())
}
