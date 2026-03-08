use zparse::{
    Config, ConvertOptions, Format, convert_with_options, detect_format_from_path, from_str,
    from_str_with_config,
};

#[test]
fn detect_format_from_path_supports_jsonc_case_insensitive() {
    assert_eq!(detect_format_from_path("file.jsonc"), Some(Format::Json));
    assert_eq!(detect_format_from_path("file.JSONC"), Some(Format::Json));
    assert_eq!(detect_format_from_path("file.JsOnC"), Some(Format::Json));
}

#[test]
fn detect_format_from_path_handles_regular_formats() {
    assert_eq!(detect_format_from_path("file.json"), Some(Format::Json));
    assert_eq!(detect_format_from_path("file.toml"), Some(Format::Toml));
    assert_eq!(detect_format_from_path("file.yaml"), Some(Format::Yaml));
    assert_eq!(detect_format_from_path("file.yml"), Some(Format::Yaml));
    assert_eq!(detect_format_from_path("file.xml"), Some(Format::Xml));
}

#[test]
fn detect_format_from_path_without_extension_returns_none() {
    assert_eq!(detect_format_from_path("jsonc"), None);
    assert_eq!(detect_format_from_path("file"), None);
}

#[test]
fn parse_jsonc_with_comments_and_trailing_commas() {
    let input = r#"{
        // line comment
        "name": "zparse",
        "arr": [1, 2, 3,],
    }"#;

    let config = Config::default()
        .with_comments(true)
        .with_trailing_commas(true);
    let value = from_str_with_config(input, config);

    assert!(value.is_ok());
}

#[test]
fn convert_jsonc_to_json_normalizes_to_strict_json() {
    let input = r#"{
        // line comment
        "name": "zparse",
        /* block comment */
        "value": 42,
        "arr": [1, 2, 3,],
    }"#;

    let options = ConvertOptions {
        json: Config::default()
            .with_comments(true)
            .with_trailing_commas(true),
        ..Default::default()
    };

    let output = convert_with_options(input, Format::Json, Format::Json, &options);
    assert!(output.is_ok());

    if let Ok(normalized) = output {
        assert!(!normalized.contains("//"));
        assert!(!normalized.contains("/*"));
        assert!(from_str(&normalized).is_ok());
    }
}

#[test]
fn convert_jsonc_to_toml_works() {
    let input = r#"{
        "name": "zparse", // comment
        "value": 42,
    }"#;
    let options = ConvertOptions {
        json: Config::default()
            .with_comments(true)
            .with_trailing_commas(true),
        ..Default::default()
    };

    let output = convert_with_options(input, Format::Json, Format::Toml, &options);
    assert!(output.is_ok());

    if let Ok(toml) = output {
        assert!(toml.contains("name"));
        assert!(toml.contains("value"));
    }
}
