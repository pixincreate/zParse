use zparse::{
    Config, ConvertOptions, Format, convert_with_options, detect_format_from_path,
    from_str_with_config, is_jsonc_path,
};

#[test]
fn detect_format_from_path_recognizes_jsonc() {
    assert_eq!(detect_format_from_path("config.jsonc"), Some(Format::Json));
    assert_eq!(
        detect_format_from_path("settings.jsonc"),
        Some(Format::Json)
    );
    assert_eq!(
        detect_format_from_path("/path/to/file.jsonc"),
        Some(Format::Json)
    );
}

#[test]
fn detect_format_from_path_case_insensitive_jsonc() {
    assert_eq!(detect_format_from_path("CONFIG.JSONC"), Some(Format::Json));
    assert_eq!(detect_format_from_path("file.Jsonc"), Some(Format::Json));
    assert_eq!(detect_format_from_path("FILE.JsOnC"), Some(Format::Json));
}

#[test]
fn is_jsonc_path_returns_true_for_jsonc_extension() {
    assert!(is_jsonc_path("config.jsonc"));
    assert!(is_jsonc_path("settings.jsonc"));
    assert!(is_jsonc_path("/absolute/path/to/file.jsonc"));
    assert!(is_jsonc_path("relative/path/file.jsonc"));
}

#[test]
fn is_jsonc_path_case_insensitive() {
    assert!(is_jsonc_path("CONFIG.JSONC"));
    assert!(is_jsonc_path("file.Jsonc"));
    assert!(is_jsonc_path("FILE.JsOnC"));
    assert!(is_jsonc_path("test.JSONC"));
}

#[test]
fn is_jsonc_path_returns_false_for_non_jsonc() {
    assert!(!is_jsonc_path("file.json"));
    assert!(!is_jsonc_path("file.txt"));
    assert!(!is_jsonc_path("file.toml"));
    assert!(!is_jsonc_path("file.yaml"));
    assert!(!is_jsonc_path("file"));
}

#[test]
fn parse_jsonc_with_line_comments() {
    let jsonc = r#"{
        "name": "test", // this is a comment
        "value": 42 // another comment
    }"#;

    let config = Config::default().with_comments(true);
    let parsed = from_str_with_config(jsonc, config);
    assert!(parsed.is_ok());

    if let Ok(value) = parsed {
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("name"))
                .and_then(|v| v.as_string()),
            Some("test")
        );
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("value"))
                .and_then(|v| v.as_number()),
            Some(42.0)
        );
    }
}

#[test]
fn parse_jsonc_with_block_comments() {
    let jsonc = r#"{
        /* block comment */
        "name": "test",
        /* another
           multiline
           comment */
        "value": 42
    }"#;

    let config = Config::default().with_comments(true);
    let parsed = from_str_with_config(jsonc, config);
    assert!(parsed.is_ok());

    if let Ok(value) = parsed {
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("name"))
                .and_then(|v| v.as_string()),
            Some("test")
        );
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("value"))
                .and_then(|v| v.as_number()),
            Some(42.0)
        );
    }
}

#[test]
fn parse_jsonc_with_mixed_comments() {
    let jsonc = r#"{
        // line comment at start
        "name": "test", /* inline block comment */
        /* block comment before field */
        "value": 42,
        "nested": { "key": "value" }
    }"#;

    let config = Config::default().with_comments(true);
    let parsed = from_str_with_config(jsonc, config);
    assert!(parsed.is_ok());

    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(3));
    }
}

#[test]
fn parse_jsonc_with_trailing_comma_in_object() {
    let jsonc = r#"{
        "name": "test",
        "value": 42,
    }"#;

    let config = Config::default().with_trailing_commas(true);
    let parsed = from_str_with_config(jsonc, config);
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(2));
    }
}

#[test]
fn parse_jsonc_with_trailing_comma_in_array() {
    let jsonc = r#"{
        "items": [1, 2, 3,]
    }"#;

    let config = Config::default().with_trailing_commas(true);
    let parsed = from_str_with_config(jsonc, config);
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        let len = value
            .as_object()
            .and_then(|obj| obj.get("items"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.len());
        assert_eq!(len, Some(3));
    }
}

#[test]
fn parse_jsonc_with_both_comments_and_trailing_commas() {
    let jsonc = r#"{
        "name": "test", // comment
        "items": [1, 2, 3,],
        "nested": { "key": "value", },
    }"#;

    let config = Config::default()
        .with_comments(true)
        .with_trailing_commas(true);
    let parsed = from_str_with_config(jsonc, config);
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(3));
    }
}

#[test]
fn convert_jsonc_to_toml() {
    let jsonc = r#"{
        "name": "test", // this is a comment
        "value": 42,
    }"#;
    let options = ConvertOptions {
        json: Config::default()
            .with_comments(true)
            .with_trailing_commas(true),
    };

    let converted = convert_with_options(jsonc, Format::Json, Format::Toml, &options);
    assert!(converted.is_ok());
    if let Ok(output) = converted {
        assert!(output.contains("name"));
        assert!(output.contains("test"));
        assert!(output.contains("value"));
        assert!(output.contains("42"));
    }
}

#[test]
fn convert_jsonc_to_yaml() {
    let jsonc = r#"{
        "name": "test", // comment
        "value": 42,
        "items": [1, 2, 3,]
    }"#;
    let options = ConvertOptions {
        json: Config::default()
            .with_comments(true)
            .with_trailing_commas(true),
    };

    let converted = convert_with_options(jsonc, Format::Json, Format::Yaml, &options);
    assert!(converted.is_ok());
    if let Ok(output) = converted {
        assert!(output.contains("name"));
        assert!(output.contains("test"));
        assert!(output.contains("value"));
        assert!(output.contains("42"));
    }
}

#[test]
fn convert_jsonc_to_xml() {
    let jsonc = r#"{
        "root": {
            "name": "test", // comment
            "value": 42
        }
    }"#;
    let options = ConvertOptions {
        json: Config::default().with_comments(true),
    };

    let converted = convert_with_options(jsonc, Format::Json, Format::Xml, &options);
    assert!(converted.is_ok());
    if let Ok(output) = converted {
        assert!(output.contains("<root>"));
        assert!(output.contains("<name>"));
        assert!(output.contains("test"));
        assert!(output.contains("<value>"));
        assert!(output.contains("42"));
    }
}

#[test]
fn convert_jsonc_to_json_strips_comments() {
    let jsonc = r#"{
        "name": "test",
        "value": 42
    }"#;
    let options = ConvertOptions {
        json: Config::default().with_comments(true),
    };

    let converted = convert_with_options(jsonc, Format::Json, Format::Json, &options);
    assert!(converted.is_ok());
    if let Ok(output) = converted {
        assert!(output.contains("name"));
        assert!(output.contains("test"));
        assert!(output.contains("value"));
        assert!(output.contains("42"));
    }
}

#[test]
fn parse_jsonc_empty_object() {
    let config = Config::default()
        .with_comments(true)
        .with_trailing_commas(true);
    let parsed = from_str_with_config("{}", config);
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(0));
    }
}

#[test]
fn parse_jsonc_empty_object_with_comments() {
    let jsonc = r#"{
        // just a comment
        /* and another */
    }"#;
    let parsed = from_str_with_config(jsonc, Config::default().with_comments(true));
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(0));
    }
}

#[test]
fn parse_jsonc_comment_only_lines() {
    let jsonc = r#"{
        // comment line 1
        // comment line 2
        "name": "test",
        // comment line 3
        "value": 42
    }"#;
    let parsed = from_str_with_config(jsonc, Config::default().with_comments(true));
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(2));
    }
}

#[test]
fn parse_jsonc_nested_structure_with_comments() {
    let jsonc = r#"{
        "outer": {
            "inner": {
                "deep": { "value": 42 }
            }
        }
    }"#;
    let parsed = from_str_with_config(jsonc, Config::default().with_comments(true));
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        let num = value
            .as_object()
            .and_then(|obj| obj.get("outer"))
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("inner"))
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("deep"))
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("value"))
            .and_then(|v| v.as_number());
        assert_eq!(num, Some(42.0));
    }
}

#[test]
fn parse_jsonc_array_with_trailing_commas() {
    let parsed = from_str_with_config("[1,2,3,]", Config::default().with_trailing_commas(true));
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_array().map(|arr| arr.len()), Some(3));
    }
}

#[test]
fn parse_jsonc_complex_real_world_example() {
    let jsonc = r#"{
        "app": {
            "name": "zparse-test",
            "features": ["parsing", "conversion", "validation",],
        },
        "database": {
            "host": "localhost",
            "port": 5432,
        },
    }"#;
    let parsed = from_str_with_config(
        jsonc,
        Config::default()
            .with_comments(true)
            .with_trailing_commas(true),
    );
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(2));
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("app"))
                .and_then(|v| v.as_object())
                .and_then(|obj| obj.get("name"))
                .and_then(|v| v.as_string()),
            Some("zparse-test")
        );
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("app"))
                .and_then(|v| v.as_object())
                .and_then(|obj| obj.get("features"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.len()),
            Some(3)
        );
    }
}

#[test]
fn parse_jsonc_with_escaped_strings() {
    let jsonc = r#"{
        "greeting": "Hello, World!",
        "special": "tab\there",
        "quote": "say \"hello\""
    }"#;
    let parsed = from_str_with_config(jsonc, Config::default().with_comments(true));
    assert!(parsed.is_ok());
    if let Ok(value) = parsed {
        assert_eq!(
            value
                .as_object()
                .and_then(|obj| obj.get("greeting"))
                .and_then(|v| v.as_string()),
            Some("Hello, World!")
        );
        assert!(
            value
                .as_object()
                .map(|obj| obj.contains_key("special"))
                .unwrap_or(false)
        );
        assert!(
            value
                .as_object()
                .map(|obj| obj.contains_key("quote"))
                .unwrap_or(false)
        );
    }
}
