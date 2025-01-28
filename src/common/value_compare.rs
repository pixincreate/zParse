use crate::parser::Value;

pub fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Object(l_map), Value::Object(r_map))
        | (Value::Table(l_map), Value::Table(r_map))
        | (Value::Object(l_map), Value::Table(r_map))
        | (Value::Table(l_map), Value::Object(r_map)) => {
            if l_map.len() != r_map.len() {
                return false;
            }
            l_map
                .iter()
                .all(|(k, v)| r_map.get(k).is_some_and(|r_v| values_equal(v, r_v)))
        }
        (Value::Array(l_arr), Value::Array(r_arr)) => {
            if l_arr.len() != r_arr.len() {
                return false;
            }
            l_arr
                .iter()
                .zip(r_arr.iter())
                .all(|(l, r)| values_equal(l, r))
        }
        (Value::Number(l), Value::Number(r)) => (l - r).abs() < f64::EPSILON,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Null, Value::Null) => true,
        (Value::DateTime(l), Value::DateTime(r)) => l == r,
        _ => false,
    }
}
