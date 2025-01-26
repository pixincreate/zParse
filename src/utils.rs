use crate::{
    error::{ParseError, ParseErrorKind, Result},
    parser::{json::JsonParser, toml::TomlParser, value::Value},
};
use std::{collections::HashMap, fs};

pub fn read_file(path: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|_| {
        ParseError::new(ParseErrorKind::IoError(format!(
            "Cannot read file: {}",
            path
        )))
    })
}

pub fn write_file(path: &str, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|_| {
        ParseError::new(ParseErrorKind::IoError(format!(
            "Cannot write to file: {}",
            path
        )))
    })
}

pub fn parse_json(content: &str) -> Result<Value> {
    let mut parser = JsonParser::new(content)?;
    parser.parse()
}

pub fn parse_toml(content: &str) -> Result<Value> {
    let mut parser = TomlParser::new(content)?;
    parser.parse()
}

pub fn format_json(value: &Value) -> String {
    format_value(value, 0)
}

pub fn format_toml(value: &Value) -> String {
    match value {
        Value::Table(map) => format_toml_table(map, Vec::new()),
        _ => format_value(value, 0),
    }
}

fn format_value(value: &Value, indent: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::DateTime(dt) => dt.to_string(),
        Value::Array(arr) => format_array(arr, indent),
        Value::Object(map) | Value::Table(map) => format_object(map, indent),
    }
}

fn format_object(map: &HashMap<String, Value>, indent: usize) -> String {
    if map.is_empty() {
        return "{}".to_string();
    }

    let indent_str = " ".repeat(indent);
    let inner_indent = " ".repeat(indent + 2);

    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    let items: Vec<String> = entries
        .iter()
        .map(|(k, v)| format!("{}\"{}\": {}", inner_indent, k, format_value(v, indent + 2)))
        .collect();

    format!("{{\n{}\n{}}}", items.join(",\n"), indent_str)
}

pub fn format_toml_table(map: &HashMap<String, Value>, path: Vec<String>) -> String {
    let mut result = String::new();
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    // Process simple key-value pairs first
    for (key, value) in entries.iter() {
        match value {
            Value::Table(_) | Value::Array(_) => continue, // Handle tables and arrays later
            _ => {
                result.push_str(&format!("{} = {}\n", key, format_value(value, 0)));
            }
        }
    }

    // Process array tables
    for (key, value) in entries.iter() {
        if let Value::Array(arr) = value {
            if arr
                .iter()
                .any(|v| matches!(v, Value::Table(_) | Value::Object(_)))
            {
                for item in arr {
                    if let Value::Table(inner_map) | Value::Object(inner_map) = item {
                        let mut new_path = path.clone();
                        new_path.push(key.to_string());
                        result.push_str(&format!("\n[[{}]]\n", new_path.join(".")));

                        // Sort and process inner map entries
                        let mut inner_entries: Vec<_> = inner_map.iter().collect();
                        inner_entries.sort_by_key(|(k, _)| *k);

                        // Process non-table entries first
                        for (k, v) in inner_entries.iter() {
                            if !matches!(v, Value::Table(_) | Value::Object(_) | Value::Array(_)) {
                                result.push_str(&format!("{} = {}\n", k, format_value(v, 0)));
                            }
                        }

                        // Process nested array tables
                        for (k, v) in inner_entries {
                            if let Value::Array(nested_arr) = v {
                                if nested_arr
                                    .iter()
                                    .any(|v| matches!(v, Value::Table(_) | Value::Object(_)))
                                {
                                    let mut nested_path = new_path.clone();
                                    nested_path.push(k.to_string());
                                    for nested_item in nested_arr {
                                        if let Value::Table(nested_map)
                                        | Value::Object(nested_map) = nested_item
                                        {
                                            result.push_str(&format!(
                                                "\n  [[{}]]\n",
                                                nested_path.join(".")
                                            ));
                                            for (nested_k, nested_v) in nested_map {
                                                result.push_str(&format!(
                                                    "  {} = {}\n",
                                                    nested_k,
                                                    format_value(nested_v, 0)
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                result.push_str(&format!("{} = {}\n", key, format_array(arr, 0)));
            }
        }
    }

    // Process regular tables
    for (key, value) in entries.iter() {
        if let Value::Table(inner_map) = value {
            if !result.is_empty() {
                result.push('\n');
            }
            let mut new_path = path.clone();
            new_path.push(key.to_string());
            result.push_str(&format!("[{}]\n", new_path.join(".")));
            result.push_str(&format_toml_table(inner_map, new_path));
        }
    }

    result
}

fn format_array(arr: &[Value], indent: usize) -> String {
    if arr.is_empty() {
        return "[]".to_string();
    }

    let indent_str = " ".repeat(indent);
    let inner_indent = " ".repeat(indent + 2);

    let items: Vec<String> = arr
        .iter()
        .map(|v| format!("{}{}", inner_indent, format_value(v, indent + 2)))
        .collect();

    format!("[\n{}\n{}]", items.join(",\n"), indent_str)
}
