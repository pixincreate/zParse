use std::fs;

use zparse::yaml::Parser;

#[test]
fn test_valid_yaml_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let valid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/yaml/valid");
    for entry in fs::read_dir(valid_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let mut parser = Parser::new(content.as_bytes());
        let value = parser.parse();
        if value.is_err() {
            return Err(std::io::Error::other(format!(
                "Failed to parse valid YAML fixture: {path:?}"
            ))
            .into());
        }
    }
    Ok(())
}

#[test]
fn test_invalid_yaml_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let invalid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/yaml/invalid");
    for entry in fs::read_dir(invalid_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let mut parser = Parser::new(content.as_bytes());
        let value = parser.parse();
        if value.is_ok() {
            return Err(std::io::Error::other(format!(
                "Invalid YAML fixture parsed successfully: {path:?}"
            ))
            .into());
        }
    }
    Ok(())
}
