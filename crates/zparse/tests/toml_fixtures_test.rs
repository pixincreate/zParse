use std::fs;

use zparse::toml::Parser;

#[test]
fn test_valid_toml_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let valid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/toml/valid");
    for entry in fs::read_dir(valid_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let mut parser = Parser::new(content.as_bytes());
        let value = parser.parse();
        if value.is_err() {
            return Err(std::io::Error::other(format!(
                "Failed to parse valid TOML fixture: {path:?}"
            ))
            .into());
        }
    }
    Ok(())
}

#[test]
fn test_invalid_toml_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let invalid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/toml/invalid");
    for entry in fs::read_dir(invalid_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let mut parser = Parser::new(content.as_bytes());
        let value = parser.parse();
        if value.is_ok() {
            return Err(std::io::Error::other(format!(
                "Invalid TOML fixture parsed successfully: {path:?}"
            ))
            .into());
        }
    }
    Ok(())
}
