use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use zparse::yaml::Parser;
use zparse::{Object, Value};

fn parse_or_fail(input: &str) -> Result<Value, TestCaseError> {
    let mut parser = Parser::new(input.as_bytes());
    parser
        .parse()
        .map_err(|err| TestCaseError::fail(format!("parse failed: {err}")))
}

fn ensure_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), TestCaseError> {
    if left == right {
        Ok(())
    } else {
        Err(TestCaseError::fail(format!(
            "assertion failed: left={left:?} right={right:?}"
        )))
    }
}

fn serialize_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", escape_string(s)),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(serialize_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(obj) => serialize_flow_mapping(obj),
        Value::Datetime(_) => "null".to_string(),
    }
}

fn serialize_mapping(obj: &Object) -> String {
    let mut lines = Vec::new();
    for (key, value) in obj.iter() {
        lines.push(format!("{key}: {}", serialize_value(value)));
    }
    lines.join("\n")
}

fn serialize_flow_mapping(obj: &Object) -> String {
    let mut entries = Vec::new();
    for (key, value) in obj.iter() {
        entries.push(format!("{key}: {}", serialize_value(value)));
    }
    format!("{{{}}}", entries.join(", "))
}

fn escape_string(input: &str) -> String {
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

fn arb_key() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(|s| s)
}

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i32>().prop_map(|n| Value::Number(f64::from(n))),
        "[a-zA-Z_][a-zA-Z0-9_]*"
            .prop_filter("avoid yaml keywords", |s| {
                !matches!(
                    s.as_str(),
                    "null"
                        | "Null"
                        | "NULL"
                        | "true"
                        | "True"
                        | "TRUE"
                        | "false"
                        | "False"
                        | "FALSE"
                )
            })
            .prop_map(Value::String),
    ];

    leaf.prop_recursive(4, 64, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(|v| Value::Array(v.into())),
            prop::collection::hash_map(arb_key(), inner, 0..6).prop_map(|map| {
                let obj: Object = map.into_iter().collect();
                Value::Object(obj)
            }),
        ]
    })
}

proptest! {
    #[test]
    fn yaml_roundtrip(obj in prop::collection::hash_map(arb_key(), arb_value(), 1..6)) {
        let obj: Object = obj.into_iter().collect();
        let yaml = serialize_mapping(&obj);
        let parsed = parse_or_fail(&yaml)?;

        if let Value::Object(parsed_obj) = parsed {
            ensure_eq(parsed_obj, obj)?;
        } else {
            return Err(TestCaseError::fail("expected object"));
        }
    }
}
