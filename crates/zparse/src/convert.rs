//! Format conversion utilities

use crate::csv::infer_primitive_value;
use crate::csv::parser::Config as CsvConfig;
use crate::csv::Parser as CsvParser;
use crate::error::{Error, ErrorKind, Result, Span};
use crate::json::{Config as JsonConfig, Parser as JsonParser};
use crate::toml::Parser as TomlParser;
use crate::value::{Array, Object, TomlDatetime, Value};
use crate::xml::model::{Content as XmlContent, Document as XmlDocument, Element as XmlElement};
use crate::xml::parser::Parser as XmlParser;
use crate::yaml::Parser as YamlParser;
use indexmap::IndexMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Csv,
    Toml,
    Yaml,
    Xml,
}

/// Conversion options per format
#[derive(Clone, Debug, Default)]
pub struct ConvertOptions {
    pub json: JsonConfig,
    pub csv: CsvConfig,
}

/// Convert between supported formats
pub fn convert(input: &str, from: Format, to: Format) -> Result<String> {
    convert_with_options(input, from, to, &ConvertOptions::default())
}

/// Convert between supported formats with options
pub fn convert_with_options(
    input: &str,
    from: Format,
    to: Format,
    options: &ConvertOptions,
) -> Result<String> {
    if from == to {
        if from == Format::Json
            && (options.json.allow_comments || options.json.allow_trailing_commas)
        {
            let value = parse_value(input, from, options)?;
            return serialize_value(&value, to, options);
        }
        return Ok(input.to_string());
    }

    match (from, to) {
        (Format::Csv, Format::Xml) => {
            let value = parse_value(input, from, options)?;
            let doc = csv_value_to_xml(&value)?;
            Ok(serialize_xml(&doc))
        }
        (Format::Xml, Format::Csv) => {
            let mut parser = XmlParser::new(input.as_bytes());
            let doc = parser.parse()?;
            let value = xml_to_csv_value(&doc)?;
            serialize_value(&value, to, options)
        }
        (Format::Xml, _) => {
            let mut parser = XmlParser::new(input.as_bytes());
            let doc = parser.parse()?;
            let value = xml_to_value(&doc);
            serialize_value(&value, to, options)
        }
        (_, Format::Xml) => {
            let value = parse_value(input, from, options)?;
            let doc = value_to_xml(&value);
            Ok(serialize_xml(&doc))
        }
        _ => {
            let value = parse_value(input, from, options)?;
            let value = normalize_for_target(value, from, to);
            serialize_value(&value, to, options)
        }
    }
}

fn normalize_for_target(value: Value, from: Format, to: Format) -> Value {
    match (from, to, value) {
        (Format::Csv, Format::Toml, Value::Array(rows)) => {
            let mut root = Object::new();
            root.insert("rows", Value::Array(rows));
            Value::Object(root)
        }
        (_, _, value) => value,
    }
}

fn parse_value(input: &str, format: Format, options: &ConvertOptions) -> Result<Value> {
    match format {
        Format::Json => {
            let mut parser = JsonParser::with_config(input.as_bytes(), options.json);
            parser.parse_value()
        }
        Format::Csv => {
            let mut parser = CsvParser::with_config(input.as_bytes(), options.csv);
            parser.parse()
        }
        Format::Toml => {
            let mut parser = TomlParser::new(input.as_bytes());
            parser.parse()
        }
        Format::Yaml => {
            let mut parser = YamlParser::new(input.as_bytes());
            parser.parse()
        }
        Format::Xml => Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "xml requires xml parser".to_string(),
        )),
    }
}

fn serialize_value(value: &Value, format: Format, options: &ConvertOptions) -> Result<String> {
    match format {
        Format::Json => Ok(serialize_json(value)),
        Format::Csv => serialize_csv(value, options.csv.delimiter),
        Format::Toml => serialize_toml(value),
        Format::Yaml => Ok(serialize_yaml(value, 0)),
        Format::Xml => Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "xml requires xml serializer".to_string(),
        )),
    }
}

fn serialize_csv(value: &Value, delimiter: u8) -> Result<String> {
    // Safe: delimiter is validated ASCII (u8 to char)
    #[allow(clippy::as_conversions)]
    let delim_char = delimiter as char;
    let delim_str = delim_char.to_string();
    let mut owned_rows = Array::new();
    let rows = match value {
        Value::Array(rows) => rows,
        Value::Object(obj) => {
            if let Some(Value::Array(rows)) = obj.get("rows") {
                rows
            } else {
                owned_rows.push(Value::Object(obj.clone()));
                &owned_rows
            }
        }
        _ => {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "csv output requires array or object root".to_string(),
            ));
        }
    };

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut headers = Vec::new();
    for row in rows {
        let obj = row.as_object().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "csv output requires array of objects".to_string(),
            )
        })?;

        for key in obj.keys() {
            if !headers.iter().any(|header| header == key) {
                headers.push(key.clone());
            }
        }
    }

    if headers.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::new();
    output.push_str(
        &headers
            .iter()
            .map(|header| escape_csv(header, delimiter))
            .collect::<Vec<_>>()
            .join(&delim_str),
    );
    output.push('\n');

    for row in rows {
        let obj = row.as_object().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "csv output requires array of objects".to_string(),
            )
        })?;

        let fields: Vec<String> = headers
            .iter()
            .map(|header| {
                let value = obj.get(header).unwrap_or(&Value::Null);
                let cell = match value {
                    Value::Null => Ok(String::new()),
                    Value::Bool(boolean) => Ok(boolean.to_string()),
                    Value::Number(number) => {
                        if number.is_finite() {
                            if number.fract() == 0.0 {
                                Ok(format!("{number:.0}"))
                            } else {
                                Ok(number.to_string())
                            }
                        } else {
                            Ok(String::new())
                        }
                    }
                    Value::String(text) => Ok(text.clone()),
                    Value::Datetime(dt) => format_datetime(dt),
                    Value::Array(_) | Value::Object(_) => Ok(serialize_json(value)),
                }?;
                let escaped = if matches!(value, Value::String(_)) {
                    escape_csv_force_quoted(&cell)
                } else {
                    escape_csv(&cell, delimiter)
                };
                Ok::<String, Error>(escaped)
            })
            .collect::<Result<Vec<_>>>()?;

        output.push_str(&fields.join(&delim_str));
        output.push('\n');
    }

    Ok(output)
}

fn escape_csv(input: &str, delimiter: u8) -> String {
    // Safe: delimiter is validated ASCII (u8 to char)
    #[allow(clippy::as_conversions)]
    let delim_char = delimiter as char;
    if input.contains(delim_char)
        || input.contains('"')
        || input.contains('\n')
        || input.contains('\r')
    {
        format!("\"{}\"", input.replace('"', "\"\""))
    } else {
        input.to_string()
    }
}

fn escape_csv_force_quoted(input: &str) -> String {
    format!("\"{}\"", input.replace('"', "\"\""))
}

fn serialize_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_finite() {
                n.to_string()
            } else {
                "null".to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", escape_json(s)),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(serialize_json).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\":{}", escape_json(k), serialize_json(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Datetime(dt) => format!(
            "\"{}\"",
            format_datetime(dt).unwrap_or_else(|_| "null".to_string())
        ),
    }
}

fn escape_string(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0C}' => result.push_str("\\f"),
            #[allow(clippy::as_conversions)]
            ch if u32::from(ch) < 0x20 => {
                result.push_str(&format!("\\u{:04x}", u32::from(ch)));
            }
            _ => result.push(ch),
        }
    }
    result
}

fn escape_json(input: &str) -> String {
    escape_string(input)
}

fn serialize_toml(value: &Value) -> Result<String> {
    match value {
        Value::Object(obj) => Ok(serialize_toml_object(obj)),
        _ => Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "toml root must be object".to_string(),
        )),
    }
}

fn serialize_toml_object(obj: &Object) -> String {
    let mut lines = Vec::new();
    for (key, value) in obj.iter() {
        let escaped_key = escape_toml_key(key);
        lines.push(format!("{escaped_key} = {}", serialize_toml_value(value)));
    }
    lines.join("\n")
}

fn serialize_toml_value(value: &Value) -> String {
    match value {
        Value::Null => "\"\"".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_finite() {
                n.to_string()
            } else {
                "nan".to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", escape_toml(s)),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(serialize_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(obj) => {
            let entries: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{} = {}", escape_toml_key(k), serialize_toml_value(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        Value::Datetime(dt) => {
            format_datetime(dt).unwrap_or_else(|_| "\"\"\"1979-05-27T07:32:00\"\"\"".to_string())
        }
    }
}

fn csv_value_to_xml(value: &Value) -> Result<XmlDocument> {
    let rows = value.as_array().ok_or_else(|| {
        Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "csv value must be an array of objects for xml conversion".to_string(),
        )
    })?;

    let mut children = Vec::new();
    for row in rows {
        let obj = row.as_object().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "csv row must be an object".to_string(),
            )
        })?;

        let mut row_children = Vec::new();
        for (key, value) in obj.iter() {
            let element = XmlElement {
                name: key.clone(),
                attributes: IndexMap::new(),
                children: value_to_children(value),
            };
            row_children.push(XmlContent::Element(element));
        }

        children.push(XmlContent::Element(XmlElement {
            name: "row".to_string(),
            attributes: IndexMap::new(),
            children: row_children,
        }));
    }

    Ok(XmlDocument {
        root: XmlElement {
            name: "root".to_string(),
            attributes: IndexMap::new(),
            children,
        },
    })
}

fn xml_to_csv_value(doc: &XmlDocument) -> Result<Value> {
    let mut rows = Array::new();

    for child in &doc.root.children {
        let XmlContent::Element(row_element) = child else {
            continue;
        };

        if row_element.name != "row" {
            continue;
        }

        let mut row = Object::new();
        for row_child in &row_element.children {
            if let XmlContent::Element(field) = row_child {
                row.insert(field.name.clone(), xml_leaf_to_value(field)?);
            }
        }

        rows.push(Value::Object(row));
    }

    Ok(Value::Array(rows))
}

fn xml_leaf_to_value(element: &XmlElement) -> Result<Value> {
    if element.children.is_empty() {
        return Ok(Value::Null);
    }

    if element.children.len() == 1 {
        if let Some(XmlContent::Text(text)) = element.children.first() {
            let trimmed = text.trim();
            return Ok(
                infer_primitive_value(trimmed).unwrap_or_else(|| Value::String(text.clone()))
            );
        }
    }

    Err(Error::with_message(
        ErrorKind::InvalidToken,
        Span::empty(),
        "xml row fields must be simple leaf elements".to_string(),
    ))
}

fn escape_toml(input: &str) -> String {
    escape_string(input)
}

fn escape_toml_key(key: &str) -> String {
    // Keys containing special chars must be quoted in TOML
    // Special chars: space, tab, ., [, ], =, ", ', #
    if key
        .chars()
        .any(|c| matches!(c, ' ' | '\t' | '.' | '[' | ']' | '=' | '"' | '\'' | '#'))
    {
        format!("\"{}\"", key.replace('"', "\\\""))
    } else {
        key.to_string()
    }
}

fn serialize_yaml(value: &Value, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match value {
        Value::Null => format!("{pad}null"),
        Value::Bool(b) => format!("{pad}{b}"),
        Value::Number(n) => format!("{pad}{n}"),
        Value::String(s) => format!("{pad}\"{}\"", escape_yaml(s)),
        Value::Datetime(dt) => format!(
            "{pad}{}",
            format_datetime(dt).unwrap_or_else(|_| "null".to_string())
        ),
        Value::Array(arr) => arr
            .iter()
            .map(|v| {
                let item = serialize_yaml(v, indent + 2);
                format!("{pad}- {}", item.trim_start())
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(obj) => obj
            .iter()
            .map(|(k, v)| {
                let value = serialize_yaml(v, indent + 2);
                if matches!(v, Value::Array(_) | Value::Object(_)) {
                    format!("{pad}{k}:\n{value}")
                } else {
                    format!("{pad}{k}: {}", value.trim_start())
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn escape_yaml(input: &str) -> String {
    escape_string(input)
}

fn format_datetime(dt: &TomlDatetime) -> Result<String> {
    use time::format_description::well_known::Rfc3339;
    use time::macros::format_description;
    match dt {
        TomlDatetime::OffsetDateTime(value) => value.format(&Rfc3339).map_err(|e| {
            Error::with_message(
                ErrorKind::InvalidDatetime,
                Span::empty(),
                format!("failed to format OffsetDateTime: {e}"),
            )
        }),
        TomlDatetime::LocalDateTime(value) => value
            .format(&format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second]"
            ))
            .map_err(|e| {
                Error::with_message(
                    ErrorKind::InvalidDatetime,
                    Span::empty(),
                    format!("failed to format LocalDateTime: {e}"),
                )
            }),
        TomlDatetime::LocalDate(value) => value
            .format(&format_description!("[year]-[month]-[day]"))
            .map_err(|e| {
                Error::with_message(
                    ErrorKind::InvalidDatetime,
                    Span::empty(),
                    format!("failed to format LocalDate: {e}"),
                )
            }),
        TomlDatetime::LocalTime(value) => value
            .format(&format_description!("[hour]:[minute]:[second]"))
            .map_err(|e| {
                Error::with_message(
                    ErrorKind::InvalidDatetime,
                    Span::empty(),
                    format!("failed to format LocalTime: {e}"),
                )
            }),
    }
}

fn xml_to_value(doc: &XmlDocument) -> Value {
    let mut root = Object::new();
    root.insert(&doc.root.name, element_to_value(&doc.root));
    Value::Object(root)
}

fn element_to_value(element: &XmlElement) -> Value {
    let mut obj = Object::new();

    if !element.attributes.is_empty() {
        let mut attrs = Object::new();
        for (key, value) in element.attributes.iter() {
            attrs.insert(key, value.clone());
        }
        obj.insert("@attributes", Value::Object(attrs));
    }

    let mut text = String::new();
    for child in &element.children {
        if let XmlContent::Text(value) = child {
            text.push_str(value);
        }
    }
    if !text.trim().is_empty() {
        obj.insert("#text", Value::String(text));
    }

    for child in &element.children {
        if let XmlContent::Element(child) = child {
            let value = element_to_value(child);
            if let Some(existing) = obj.get_mut(&child.name) {
                match existing {
                    Value::Array(items) => {
                        items.push(value);
                    }
                    _ => {
                        let previous = std::mem::replace(existing, Value::Null);
                        *existing = Value::Array(vec![previous, value].into());
                    }
                }
            } else {
                obj.insert(&child.name, value);
            }
        }
    }

    if obj.is_empty() {
        Value::Null
    } else {
        Value::Object(obj)
    }
}

fn value_to_xml(value: &Value) -> XmlDocument {
    // If value is an Object with a single key, use that key as root name
    // This prevents double-wrapping on XML→Value→XML round-trips
    let (root_name, children) = match value {
        Value::Object(obj) => {
            if let Some((key, val)) = obj.iter().next() {
                if obj.len() == 1 {
                    (key.clone(), value_to_children(val))
                } else {
                    ("root".to_string(), value_to_children(value))
                }
            } else {
                ("root".to_string(), value_to_children(value))
            }
        }
        _ => ("root".to_string(), value_to_children(value)),
    };

    let root = XmlElement {
        name: root_name,
        attributes: IndexMap::new(),
        children,
    };
    XmlDocument { root }
}

fn value_to_children(value: &Value) -> Vec<XmlContent> {
    match value {
        Value::Object(obj) => obj
            .iter()
            .flat_map(|(key, value)| value_to_elements(key, value))
            .map(XmlContent::Element)
            .collect(),
        Value::Array(arr) => arr.iter().flat_map(value_to_children).collect(),
        Value::String(text) => vec![XmlContent::Text(text.clone())],
        Value::Number(n) => vec![XmlContent::Text(n.to_string())],
        Value::Bool(b) => vec![XmlContent::Text(b.to_string())],
        Value::Null => Vec::new(),
        Value::Datetime(dt) => vec![XmlContent::Text(
            format_datetime(dt).unwrap_or_else(|_| "1979-05-27T07:32:00Z".to_string()),
        )],
    }
}

fn value_to_elements(name: &str, value: &Value) -> Vec<XmlElement> {
    match value {
        Value::Array(arr) => arr
            .iter()
            .flat_map(|value| value_to_elements(name, value))
            .collect(),
        Value::Object(obj) => {
            let mut attributes = IndexMap::new();
            let mut children = Vec::new();

            if let Some(Value::Object(attrs)) = obj.get("@attributes") {
                for (key, value) in attrs.iter() {
                    if let Value::String(text) = value {
                        attributes.insert(key.clone(), text.clone());
                    } else {
                        attributes.insert(key.clone(), serialize_json(value));
                    }
                }
            }

            if let Some(Value::String(text)) = obj.get("#text") {
                children.push(XmlContent::Text(text.clone()));
            }

            for (key, value) in obj.iter() {
                if key == "@attributes" || key == "#text" {
                    continue;
                }
                for element in value_to_elements(key, value) {
                    children.push(XmlContent::Element(element));
                }
            }

            vec![XmlElement {
                name: name.to_string(),
                attributes,
                children,
            }]
        }
        _ => vec![XmlElement {
            name: name.to_string(),
            attributes: IndexMap::new(),
            children: value_to_children(value),
        }],
    }
}

fn serialize_xml(doc: &XmlDocument) -> String {
    let mut output = String::new();
    serialize_element(&doc.root, &mut output);
    output
}

fn serialize_element(element: &XmlElement, output: &mut String) {
    output.push('<');
    output.push_str(&element.name);

    for (key, value) in element.attributes.iter() {
        output.push(' ');
        output.push_str(key);
        output.push_str("=\"");
        output.push_str(&escape_xml(value));
        output.push('"');
    }

    if element.children.is_empty() {
        output.push_str("/>");
        return;
    }

    output.push('>');
    for child in &element.children {
        match child {
            XmlContent::Element(child) => serialize_element(child, output),
            XmlContent::Text(text) => output.push_str(&escape_xml(text)),
        }
    }
    output.push_str("</");
    output.push_str(&element.name);
    output.push('>');
}

fn escape_xml(input: &str) -> String {
    // Single-pass XML escaping to avoid multiple allocations
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(c),
        }
    }
    output
}
