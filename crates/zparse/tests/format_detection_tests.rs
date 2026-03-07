use zparse::{Format, detect_format_from_path};

#[test]
fn detect_format_from_path_supports_extensions() {
    assert_eq!(detect_format_from_path("input.json"), Some(Format::Json));
    assert_eq!(detect_format_from_path("input.JSON"), Some(Format::Json));
    assert_eq!(detect_format_from_path("input.toml"), Some(Format::Toml));
    assert_eq!(detect_format_from_path("input.yaml"), Some(Format::Yaml));
    assert_eq!(detect_format_from_path("input.yml"), Some(Format::Yaml));
    assert_eq!(detect_format_from_path("input.xml"), Some(Format::Xml));
    assert_eq!(detect_format_from_path("input.csv"), Some(Format::Csv));
    assert_eq!(detect_format_from_path("input.jsonc"), Some(Format::Json));
}

#[test]
fn detect_format_from_path_returns_none_for_unknown_or_missing_extensions() {
    assert_eq!(detect_format_from_path("input"), None);
    assert_eq!(detect_format_from_path("input.txt"), None);
}
