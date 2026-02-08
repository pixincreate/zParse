use std::fs;
use zparse::from_str;

#[test]
fn test_valid_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let valid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/valid");
    for entry in fs::read_dir(valid_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let result = from_str(&content);
        if result.is_err() {
            return Err(
                std::io::Error::other(format!("Failed to parse valid file: {path:?}")).into(),
            );
        }
    }
    Ok(())
}

#[test]
fn test_invalid_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let invalid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/invalid");
    for entry in fs::read_dir(invalid_dir)? {
        let entry = entry?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        let result = from_str(&content);
        if result.is_ok() {
            return Err(std::io::Error::other(format!(
                "Should fail to parse invalid file: {path:?}"
            ))
            .into());
        }
    }
    Ok(())
}
