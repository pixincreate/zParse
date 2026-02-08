//! Format conversion utilities

use crate::error::{Error, ErrorKind, Result, Span};
use crate::json::{Config as JsonConfig, Parser as JsonParser};
use crate::toml::Parser as TomlParser;
use crate::value::{Object, TomlDatetime, Value};
use crate::xml::model::{Content as XmlContent, Document as XmlDocument, Element as XmlElement};
use crate::xml::parser::Parser as XmlParser;
use crate::yaml::Parser as YamlParser;
use indexmap::IndexMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Toml,
    Yaml,
    Xml,
}

/// Conversion options per format
#[derive(Clone, Debug, Default)]
pub struct ConvertOptions {
    pub json: JsonConfig,
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
        return Ok(input.to_string());
    }

    match (from, to) {
        (Format::Xml, _) => {
            let mut parser = XmlParser::new(input.as_bytes());
            let doc = parser.parse()?;
            let value = xml_to_value(&doc);
            serialize_value(&value, to)
        }
        (_, Format::Xml) => {
            let value = parse_value(input, from, options)?;
            let doc = value_to_xml(&value);
            Ok(serialize_xml(&doc))
        }
        _ => {
            let value = parse_value(input, from, options)?;
            serialize_value(&value, to)
        }
    }
}

fn parse_value(input: &str, format: Format, options: &ConvertOptions) -> Result<Value> {
    match format {
        Format::Json => {
            let mut parser = JsonParser::with_config(input.as_bytes(), options.json);
            parser.parse_value()
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

fn serialize_value(value: &Value, format: Format) -> Result<String> {
    match format {
        Format::Json => Ok(serialize_json(value)),
        Format::Toml => serialize_toml(value),
        Format::Yaml => Ok(serialize_yaml(value, 0)),
        Format::Xml => Err(Error::with_message(
            ErrorKind::InvalidToken,
            Span::empty(),
            "xml requires xml serializer".to_string(),
        )),
    }
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
        Value::Datetime(dt) => format!("\"{}\"", format_datetime(dt)),
    }
}

fn escape_json(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
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
        lines.push(format!("{key} = {}", serialize_toml_value(value)));
    }
    lines.join("\n")
}

fn serialize_toml_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
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
                .map(|(k, v)| format!("{k} = {}", serialize_toml_value(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        Value::Datetime(dt) => format_datetime(dt),
    }
}

fn escape_toml(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn serialize_yaml(value: &Value, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match value {
        Value::Null => format!("{pad}null"),
        Value::Bool(b) => format!("{pad}{b}"),
        Value::Number(n) => format!("{pad}{n}"),
        Value::String(s) => format!("{pad}\"{}\"", escape_yaml(s)),
        Value::Datetime(dt) => format!("{pad}{}", format_datetime(dt)),
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
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn format_datetime(dt: &TomlDatetime) -> String {
    use time::format_description::well_known::Rfc3339;
    use time::macros::format_description;
    match dt {
        TomlDatetime::OffsetDateTime(value) => value
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1979-05-27T07:32:00Z".to_string()),
        TomlDatetime::LocalDateTime(value) => value
            .format(&format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second]"
            ))
            .unwrap_or_else(|_| "1979-05-27T07:32:00".to_string()),
        TomlDatetime::LocalDate(value) => value
            .format(&format_description!("[year]-[month]-[day]"))
            .unwrap_or_else(|_| "1979-05-27".to_string()),
        TomlDatetime::LocalTime(value) => value
            .format(&format_description!("[hour]:[minute]:[second]"))
            .unwrap_or_else(|_| "07:32:00".to_string()),
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
            match obj.get(&child.name) {
                Some(Value::Array(arr)) => {
                    let mut items = arr.clone();
                    items.push(value);
                    obj.insert(&child.name, Value::Array(items));
                }
                Some(existing) => {
                    let items = vec![existing.clone(), value];
                    obj.insert(&child.name, Value::Array(items.into()));
                }
                None => {
                    obj.insert(&child.name, value);
                }
            }
        }
    }

    if obj.is_empty() {
        Value::Object(Object::new())
    } else {
        Value::Object(obj)
    }
}

fn value_to_xml(value: &Value) -> XmlDocument {
    let root = XmlElement {
        name: "root".to_string(),
        attributes: IndexMap::new(),
        children: value_to_children(value),
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
        Value::Datetime(dt) => vec![XmlContent::Text(format_datetime(dt))],
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
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
