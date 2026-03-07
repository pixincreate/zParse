pub mod parser;

pub use parser::Parser;

use crate::value::Value;

pub(crate) fn infer_primitive_value(input: &str) -> Option<Value> {
    if input.is_empty() || input.eq_ignore_ascii_case("null") {
        return Some(Value::Null);
    }

    if input.eq_ignore_ascii_case("true") {
        return Some(Value::Bool(true));
    }

    if input.eq_ignore_ascii_case("false") {
        return Some(Value::Bool(false));
    }

    if input.parse::<i64>().is_ok()
        && let Ok(number) = input.parse::<f64>()
        && number.is_finite()
    {
        return Some(Value::Number(number));
    }

    if let Ok(float) = input.parse::<f64>()
        && float.is_finite()
    {
        return Some(Value::Number(float));
    }

    None
}
