use zparse::Value;
use zparse::convert::{ConvertOptions, Format, convert, convert_with_options};
use zparse::json::Config as JsonConfig;

fn expect_contains(haystack: &str, needle: &str) -> Result<(), Box<dyn std::error::Error>> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(format!("missing '{needle}' in output: {haystack}").into())
    }
}

fn expect_true(cond: bool, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    if cond {
        Ok(())
    } else {
        Err(message.to_string().into())
    }
}

fn ensure_eq<T: std::cmp::PartialEq + std::fmt::Debug>(
    left: T,
    right: T,
) -> Result<(), Box<dyn std::error::Error>> {
    if left == right {
        Ok(())
    } else {
        Err(format!("{:?} != {:?}", left, right).into())
    }
}

#[test]
fn parse_basic_csv() -> Result<(), Box<dyn std::error::Error>> {
    let value = zparse::from_csv_str("name,age,active\nAlice,30,true\nBob,25,false\n")?;
    let arr = value.as_array().ok_or("expected array")?;
    if arr.len() != 2 {
        return Err(format!("expected 2 rows, got {}", arr.len()).into());
    }
    let first = arr[0].as_object().ok_or("expected object row")?;
    if first.get("name").and_then(|v| v.as_string()) != Some("Alice") {
        return Err("unexpected name value".into());
    }
    if first.get("age").and_then(|v| v.as_number()) != Some(30.0) {
        return Err("unexpected age value".into());
    }
    if first.get("active").and_then(|v| v.as_bool()) != Some(true) {
        return Err("unexpected active value".into());
    }
    Ok(())
}

#[test]
fn parse_csv_type_inference_and_nulls() -> Result<(), Box<dyn std::error::Error>> {
    let value = zparse::from_csv_str("a,b,c,d,e\n,42,2.5,false,null\n")?;
    let row = value
        .as_array()
        .and_then(|arr| arr.get(0))
        .and_then(Value::as_object)
        .ok_or("expected first object row")?;
    if !matches!(row.get("a"), Some(Value::Null)) {
        return Err("expected null in column a".into());
    }
    if row.get("b").and_then(|v| v.as_number()) != Some(42.0) {
        return Err("expected 42 in column b".into());
    }
    if row.get("c").and_then(|v| v.as_number()) != Some(2.5) {
        return Err("expected 2.5 in column c".into());
    }
    if row.get("d").and_then(|v| v.as_bool()) != Some(false) {
        return Err("expected false in column d".into());
    }
    if !matches!(row.get("e"), Some(Value::Null)) {
        return Err("expected null in column e".into());
    }
    Ok(())
}

#[test]
fn parse_csv_quoted_and_multiline_fields() -> Result<(), Box<dyn std::error::Error>> {
    let input = "name,note\n\"Alice\",\"hello, \"\"world\"\"\"\n\"Bob\",\"line1\nline2\"\n";
    let value = zparse::from_csv_str(input)?;
    let arr = value.as_array().ok_or("expected array")?;
    let first = arr
        .get(0)
        .and_then(Value::as_object)
        .ok_or("missing row 1")?;
    let second = arr
        .get(1)
        .and_then(Value::as_object)
        .ok_or("missing row 2")?;
    if first.get("note").and_then(|v| v.as_string()) != Some("hello, \"world\"") {
        return Err("unexpected quoted note value".into());
    }
    if second.get("note").and_then(|v| v.as_string()) != Some("line1\nline2") {
        return Err("unexpected multiline note value".into());
    }
    Ok(())
}

#[test]
fn parse_csv_header_only_is_empty_array() -> Result<(), Box<dyn std::error::Error>> {
    let value = zparse::from_csv_str("name,age\n")?;
    let arr = value.as_array().ok_or("expected array")?;
    expect_true(arr.is_empty(), "expected empty rows for header-only csv")
}

#[test]
fn csv_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert("name,age\nAlice,30\n", Format::Csv, Format::Json)?;
    expect_contains(&output, "\"name\":\"Alice\"")?;
    expect_contains(&output, "\"age\":30")
}

#[test]
fn csv_to_yaml() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert("name,age\nAlice,30\n", Format::Csv, Format::Yaml)?;
    expect_contains(&output, "name:")?;
    expect_contains(&output, "age:")
}

#[test]
fn csv_to_toml() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert("name,age\nAlice,30\n", Format::Csv, Format::Toml)?;
    expect_contains(&output, "rows")?;
    expect_contains(&output, "name")
}

#[test]
fn csv_to_xml() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert("name,age\nAlice,30\n", Format::Csv, Format::Xml)?;
    expect_contains(&output, "<root")?;
    expect_contains(&output, "<name>Alice</name>")?;
    expect_contains(&output, "<age>30</age>")
}

#[test]
fn json_to_csv() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert(r#"[{"name":"Alice","age":30}]"#, Format::Json, Format::Csv)?;
    expect_true(
        output.starts_with("name,age\n") || output.starts_with("age,name\n"),
        "unexpected csv header order",
    )?;
    expect_true(
        output.contains("\"Alice\",30") || output.contains("30,\"Alice\""),
        "missing csv row values",
    )
}

#[test]
fn yaml_to_csv() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert(
        "rows: [{name: Alice, age: 30}]\n",
        Format::Yaml,
        Format::Csv,
    )?;
    expect_true(
        output.starts_with("name,age\n") || output.starts_with("age,name\n"),
        "unexpected csv header order",
    )?;
    expect_true(
        output.contains("\"Alice\",30") || output.contains("30,\"Alice\""),
        "missing csv row values",
    )
}

#[test]
fn toml_to_csv() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert(
        "[[rows]]\nname = \"Alice\"\nage = 30\n",
        Format::Toml,
        Format::Csv,
    )?;
    expect_true(
        output.starts_with("name,age\n") || output.starts_with("age,name\n"),
        "unexpected csv header order",
    )?;
    expect_true(
        output.contains("\"Alice\",30") || output.contains("30,\"Alice\""),
        "missing csv row values",
    )
}

#[test]
fn xml_to_csv() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert(
        "<root><row><name>Alice</name><age>30</age></row></root>",
        Format::Xml,
        Format::Csv,
    )?;
    expect_true(
        output.starts_with("name,age\n") || output.starts_with("age,name\n"),
        "unexpected csv header order",
    )?;
    expect_true(
        output.contains("\"Alice\",30") || output.contains("30,\"Alice\""),
        "missing csv row values",
    )
}

#[test]
fn jsonc_to_csv_strips_comments_and_trailing_commas() -> Result<(), Box<dyn std::error::Error>> {
    let options = ConvertOptions {
        json: JsonConfig::default()
            .with_comments(true)
            .with_trailing_commas(true),
    };
    let input = r#"
        // comment
        [
          { "name": "Alice", "age": 30, },
        ]
    "#;
    let output = convert_with_options(input, Format::Json, Format::Csv, &options)?;
    expect_true(
        output.starts_with("name,age\n") || output.starts_with("age,name\n"),
        "unexpected csv header order",
    )?;
    expect_true(
        output.contains("\"Alice\",30") || output.contains("30,\"Alice\""),
        "missing csv row values",
    )
}

#[test]
fn csv_to_json_is_strict_json_output() -> Result<(), Box<dyn std::error::Error>> {
    let options = ConvertOptions {
        json: JsonConfig::default()
            .with_comments(true)
            .with_trailing_commas(true),
    };
    let output = convert_with_options("name,age\nAlice,30\n", Format::Csv, Format::Json, &options)?;
    expect_true(
        !output.contains("//"),
        "json output unexpectedly contains comments",
    )?;
    expect_true(
        !output.contains(",]"),
        "json output has trailing array comma",
    )?;
    expect_true(
        !output.contains(",}"),
        "json output has trailing object comma",
    )
}

#[test]
fn csv_to_xml_and_back_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let csv = "name,age\nAlice,30\nBob,40\n";
    let xml = convert(csv, Format::Csv, Format::Xml)?;
    let csv_back = convert(&xml, Format::Xml, Format::Csv)?;
    expect_contains(&xml, "<row>")?;
    expect_true(
        csv_back.starts_with("name,age\n") || csv_back.starts_with("age,name\n"),
        "unexpected csv roundtrip header",
    )
}

#[test]
fn csv_serializer_unions_headers_from_all_rows() -> Result<(), Box<dyn std::error::Error>> {
    let output = convert(r#"[{"a":1},{"b":2}]"#, Format::Json, Format::Csv)?;
    expect_true(
        output.starts_with("a,b\n") || output.starts_with("b,a\n"),
        "expected union of row headers",
    )
}

#[test]
fn json_csv_json_preserves_string_like_values() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"[{"s1":"001","s2":"true","s3":"null","s4":""}]"#;
    let csv = convert(json, Format::Json, Format::Csv)?;
    let json_back = convert(&csv, Format::Csv, Format::Json)?;
    expect_contains(&json_back, "\"s1\":\"001\"")?;
    expect_contains(&json_back, "\"s2\":\"true\"")?;
    expect_contains(&json_back, "\"s3\":\"null\"")?;
    expect_contains(&json_back, "\"s4\":\"\"")
}

#[test]
fn csv_to_toml_null_is_valid_toml() -> Result<(), Box<dyn std::error::Error>> {
    let toml = convert("name,age\nAlice,\n", Format::Csv, Format::Toml)?;
    zparse::from_toml_str(&toml)?;
    Ok(())
}

#[test]
fn csv_empty_input_returns_empty_array() -> Result<(), Box<dyn std::error::Error>> {
    let value = zparse::from_csv_str("")?;
    let arr = value.as_array().ok_or("expected array")?;
    expect_true(arr.is_empty(), "empty input should return empty array")?;
    Ok(())
}

#[test]
fn csv_only_headers_with_whitespace_crlf() -> Result<(), Box<dyn std::error::Error>> {
    let value = zparse::from_csv_str(" name , age , active \r\n")?;
    let arr = value.as_array().ok_or("expected array")?;
    expect_true(
        arr.is_empty(),
        "CSV with whitespace/CRLF should return empty array",
    )?;
    Ok(())
}

#[test]
fn csv_whitespace_only_returns_empty_array() -> Result<(), Box<dyn std::error::Error>> {
    let value = zparse::from_csv_str("   \n  \n  \n")?;
    let arr = value.as_array().ok_or("expected array")?;
    expect_true(
        arr.is_empty(),
        "whitespace-only CSV should return empty array",
    )?;
    Ok(())
}

#[test]
fn parse_csv_with_semicolon_delimiter() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name;age\nAlice;30\nBob;25";
    let value = zparse::from_csv_str_with_delimiter(data, b';')?;
    let arr = value.as_array().ok_or("expected array")?;
    ensure_eq(arr.len(), 2)?;
    let first = arr
        .get(0)
        .ok_or("missing first row")?
        .as_object()
        .ok_or("expected object")?;
    ensure_eq(first.get("name"), Some(&Value::String("Alice".to_string())))?;
    ensure_eq(first.get("age"), Some(&Value::Number(30.0)))?;
    Ok(())
}

#[test]
fn parse_csv_with_tab_delimiter() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name\tage\nCharlie\t28";
    let value = zparse::from_csv_str_with_delimiter(data, b'\t')?;
    let arr = value.as_array().ok_or("expected array")?;
    ensure_eq(arr.len(), 1)?;
    let first = arr
        .get(0)
        .ok_or("missing first row")?
        .as_object()
        .ok_or("expected object")?;
    ensure_eq(
        first.get("name"),
        Some(&Value::String("Charlie".to_string())),
    )?;
    ensure_eq(first.get("age"), Some(&Value::Number(28.0)))?;
    Ok(())
}

#[test]
fn parse_csv_with_pipe_delimiter() -> Result<(), Box<dyn std::error::Error>> {
    let data = "name|age\nDave|35";
    let value = zparse::from_csv_str_with_delimiter(data, b'|')?;
    let arr = value.as_array().ok_or("expected array")?;
    ensure_eq(arr.len(), 1)?;
    let first = arr
        .get(0)
        .ok_or("missing first row")?
        .as_object()
        .ok_or("expected object")?;
    ensure_eq(first.get("name"), Some(&Value::String("Dave".to_string())))?;
    ensure_eq(first.get("age"), Some(&Value::Number(35.0)))?;
    Ok(())
}
