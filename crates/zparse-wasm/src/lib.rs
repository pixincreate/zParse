use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Initialize the WASM module - call this once at startup
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Error type for WASM boundary
#[derive(Serialize)]
struct JsError {
    kind: String,
    message: String,
    span: Option<JsSpan>,
}

#[derive(Serialize)]
struct JsSpan {
    start: JsPos,
    end: JsPos,
}

#[derive(Serialize)]
struct JsPos {
    offset: usize,
    line: u32,
    col: u32,
}

impl From<zparse::Error> for JsError {
    fn from(e: zparse::Error) -> Self {
        let span = {
            let span = e.span();
            if span == zparse::Span::empty() {
                None
            } else {
                Some(JsSpan {
                    start: JsPos {
                        offset: span.start.offset,
                        line: span.start.line,
                        col: span.start.col,
                    },
                    end: JsPos {
                        offset: span.end.offset,
                        line: span.end.line,
                        col: span.end.col,
                    },
                })
            }
        };

        Self {
            kind: stable_error_kind(e.kind()).to_string(),
            message: e.message().to_string(),
            span,
        }
    }
}

fn stable_error_kind(kind: &zparse::ErrorKind) -> &'static str {
    match kind {
        zparse::ErrorKind::InvalidEscapeSequence => "InvalidEscapeSequence",
        zparse::ErrorKind::InvalidUnicodeEscape => "InvalidUnicodeEscape",
        zparse::ErrorKind::UnterminatedString => "UnterminatedString",
        zparse::ErrorKind::InvalidNumber => "InvalidNumber",
        zparse::ErrorKind::InvalidToken => "InvalidToken",
        zparse::ErrorKind::Expected { .. } => "Expected",
        zparse::ErrorKind::TrailingComma => "TrailingComma",
        zparse::ErrorKind::MissingComma => "MissingComma",
        zparse::ErrorKind::DuplicateKey { .. } => "DuplicateKey",
        zparse::ErrorKind::InvalidKey => "InvalidKey",
        zparse::ErrorKind::InvalidDatetime => "InvalidDatetime",
        zparse::ErrorKind::InvalidInlineTable => "InvalidInlineTable",
        zparse::ErrorKind::InvalidArray => "InvalidArray",
        zparse::ErrorKind::MaxDepthExceeded { .. } => "MaxDepthExceeded",
        zparse::ErrorKind::MaxSizeExceeded { .. } => "MaxSizeExceeded",
    }
}

impl JsError {
    fn unknown_format(format: &str) -> Self {
        Self {
            kind: "UnknownFormat".to_string(),
            message: format!("Unknown format: {}", format),
            span: None,
        }
    }
}

fn serialize_to_js<T: Serialize>(value: &T) -> JsValue {
    match serde_wasm_bindgen::to_value(value) {
        Ok(js) => js,
        // Defensive: if serialization fails (should never happen), fall back to string
        Err(_) => JsValue::from_str("serialization failed"),
    }
}

/// Convert between formats
/// - input: the input string
/// - from: source format ("json", "csv", "toml", "yaml", "xml")
/// - to: target format ("json", "csv", "toml", "yaml", "xml")
/// Returns converted string or throws error
#[wasm_bindgen]
pub fn convert(input: &str, from: &str, to: &str) -> Result<String, JsValue> {
    let from_format = parse_format(from).map_err(|e| serialize_to_js(&e))?;
    let to_format = parse_format(to).map_err(|e| serialize_to_js(&e))?;

    zparse::convert::convert(input, from_format, to_format)
        .map_err(|e| serialize_to_js(&JsError::from(e)))
}

/// Convert CSV with a custom delimiter to another format
/// - input: the CSV input string
/// - to: target format ("json", "csv", "toml", "yaml", "xml")
/// - delimiter: single ASCII character used as field separator (e.g. ";" or "\t")
/// Returns converted string or throws error
#[wasm_bindgen]
pub fn convert_csv(input: &str, to: &str, delimiter: &str) -> Result<String, JsValue> {
    let to_format = parse_format(to).map_err(|e| serialize_to_js(&e))?;

    let delim_byte = parse_csv_delimiter(delimiter).map_err(|e| serialize_to_js(&e))?;
    let csv_config = zparse::CsvConfig::default().with_delimiter(delim_byte);
    let options = zparse::ConvertOptions {
        csv: csv_config,
        ..Default::default()
    };

    zparse::convert_with_options(input, zparse::Format::Csv, to_format, &options)
        .map_err(|e| serialize_to_js(&JsError::from(e)))
}

/// Parse content to JSON
/// - content: the input string
/// - format: source format ("json", "csv", "toml", "yaml", "xml")
/// Returns JSON string or throws error
#[wasm_bindgen]
pub fn parse(content: &str, format: &str) -> Result<String, JsValue> {
    let fmt = parse_format(format).map_err(|e| serialize_to_js(&e))?;

    match fmt {
        Format::Json => zparse::convert::convert(content, Format::Json, Format::Json),
        Format::Csv => zparse::convert::convert(content, Format::Csv, Format::Json),
        Format::Toml => zparse::convert::convert(content, Format::Toml, Format::Json),
        Format::Yaml => zparse::convert::convert(content, Format::Yaml, Format::Json),
        Format::Xml => {
            return Err(serialize_to_js(&JsError::from(
                zparse::Error::with_message(
                    zparse::ErrorKind::InvalidToken,
                    zparse::Span::empty(),
                    "XML parse is not supported in parse(); use convert() instead".to_string(),
                ),
            )));
        }
    }
    .map_err(|e| serialize_to_js(&JsError::from(e)))
}

/// Detect format from file path
/// Returns format string or undefined
#[wasm_bindgen]
pub fn detect_format(path: &str) -> Option<String> {
    zparse::detect_format_from_path(path).map(|f| format!("{:?}", f).to_lowercase())
}

type Format = zparse::convert::Format;

fn parse_format(s: &str) -> Result<Format, JsError> {
    match s.to_lowercase().as_str() {
        "json" => Ok(Format::Json),
        "csv" => Ok(Format::Csv),
        "toml" => Ok(Format::Toml),
        "yaml" => Ok(Format::Yaml),
        "xml" => Ok(Format::Xml),
        _ => Err(JsError::unknown_format(s)),
    }
}

fn parse_csv_delimiter(s: &str) -> Result<u8, JsError> {
    let mut chars = s.chars();
    let ch = chars.next().ok_or_else(|| JsError {
        kind: "InvalidToken".to_string(),
        message: "CSV delimiter must be a single ASCII character".to_string(),
        span: None,
    })?;
    if chars.next().is_some() {
        return Err(JsError {
            kind: "InvalidToken".to_string(),
            message: "CSV delimiter must be a single character".to_string(),
            span: None,
        });
    }
    if !ch.is_ascii() {
        return Err(JsError {
            kind: "InvalidToken".to_string(),
            message: "CSV delimiter must be an ASCII character".to_string(),
            span: None,
        });
    }
    let byte = ch as u8;
    if matches!(byte, b'\n' | b'\r' | b'"') {
        return Err(JsError {
            kind: "InvalidToken".to_string(),
            message: format!(
                "CSV delimiter {:?} conflicts with record separators or quoting rules",
                ch
            ),
            span: None,
        });
    }
    Ok(byte)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!();

    mod convert_tests {
        use super::*;

        #[wasm_bindgen_test]
        fn json_to_toml() {
            let input = r#"{"name": "John", "age": 30}"#;
            let result = convert(input, "json", "toml");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("name"));
            assert!(output.contains("John"));
        }

        #[wasm_bindgen_test]
        fn csv_to_json() {
            let input = "name,age\nJane,20\n";
            let result = convert(input, "csv", "json");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Jane"));
            assert!(output.contains("20"));
        }

        #[wasm_bindgen_test]
        fn json_to_yaml() {
            let input = r#"{"name": "Jane", "active": true}"#;
            let result = convert(input, "json", "yaml");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("name"));
            assert!(output.contains("Jane"));
        }

        #[wasm_bindgen_test]
        fn toml_to_json() {
            let input = r#"name = "Tom"
age = 25
"#;
            let result = convert(input, "toml", "json");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Tom"));
            assert!(output.contains("25"));
        }

        #[wasm_bindgen_test]
        fn yaml_to_json() {
            let input = r#"name: Alice
active: false
"#;
            let result = convert(input, "yaml", "json");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Alice"));
        }

        #[wasm_bindgen_test]
        fn json_to_json() {
            let input = r#"{"key": "value"}"#;
            let result = convert(input, "json", "json");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), input);
        }
    }

    mod parse_tests {
        use super::*;

        #[wasm_bindgen_test]
        fn parse_json() {
            let input = r#"{"name": "Test"}"#;
            let result = parse(input, "json");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Test"));
        }

        #[wasm_bindgen_test]
        fn parse_toml() {
            let input = r#"value = 42"#;
            let result = parse(input, "toml");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("42"));
        }

        #[wasm_bindgen_test]
        fn parse_yaml() {
            let input = "key: data";
            let result = parse(input, "yaml");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("data"));
        }

        #[wasm_bindgen_test]
        fn parse_csv() {
            let input = "name,age\nSam,21\n";
            let result = parse(input, "csv");
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Sam"));
            assert!(output.contains("21"));
        }

        #[wasm_bindgen_test]
        fn parse_xml_not_supported() {
            let input = r#"<root><item>test</item></root>"#;
            let result = parse(input, "xml");
            assert!(result.is_err());
        }
    }

    mod error_tests {
        use super::*;

        #[wasm_bindgen_test]
        fn unknown_format() {
            let input = r#"{"test": true}"#;
            let result = convert(input, "invalid", "json");
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn invalid_json() {
            let input = r#"{invalid: json}"#;
            let result = convert(input, "json", "toml");
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn invalid_toml() {
            let input = r#"not valid toml"#;
            let result = convert(input, "toml", "json");
            assert!(result.is_err());
        }
    }

    mod detect_format_tests {
        use super::*;

        #[wasm_bindgen_test]
        fn detect_json() {
            assert_eq!(detect_format("file.json"), Some("json".to_string()));
        }

        #[wasm_bindgen_test]
        fn detect_csv() {
            assert_eq!(detect_format("data.csv"), Some("csv".to_string()));
        }

        #[wasm_bindgen_test]
        fn detect_toml() {
            assert_eq!(detect_format("config.toml"), Some("toml".to_string()));
        }

        #[wasm_bindgen_test]
        fn detect_yaml() {
            assert_eq!(detect_format("data.yaml"), Some("yaml".to_string()));
            assert_eq!(detect_format("data.yml"), Some("yaml".to_string()));
        }

        #[wasm_bindgen_test]
        fn detect_xml() {
            assert_eq!(detect_format("doc.xml"), Some("xml".to_string()));
        }

        #[wasm_bindgen_test]
        fn detect_unknown() {
            assert_eq!(detect_format("file.txt"), None);
            assert_eq!(detect_format("noextension"), None);
        }
    }
}
