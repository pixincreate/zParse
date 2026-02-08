use std::fs;
use zparse::from_str;

#[test]
fn test_valid_fixtures() {
    let valid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/valid");
    for entry in fs::read_dir(valid_dir).unwrap() {
        let path = entry.unwrap().path();
        let content = fs::read_to_string(&path).unwrap();
        let result = from_str(&content);
        assert!(result.is_ok(), "Failed to parse valid file: {:?}", path);
    }
}

#[test]
fn test_invalid_fixtures() {
    let invalid_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/invalid");
    for entry in fs::read_dir(invalid_dir).unwrap() {
        let path = entry.unwrap().path();
        let content = fs::read_to_string(&path).unwrap();
        let result = from_str(&content);
        assert!(
            result.is_err(),
            "Should fail to parse invalid file: {:?}",
            path
        );
    }
}
