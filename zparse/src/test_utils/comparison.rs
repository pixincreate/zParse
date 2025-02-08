use crate::Value;

pub fn compare_values(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Map(l_map), Value::Map(r_map)) => {
            if l_map.len() != r_map.len() {
                return false;
            }
            l_map
                .iter()
                .all(|(k, v)| r_map.get(k).is_some_and(|rv| compare_values(v, rv)))
        }
        (Value::Array(l_arr), Value::Array(r_arr)) => {
            if l_arr.len() != r_arr.len() {
                return false;
            }
            l_arr
                .iter()
                .zip(r_arr.iter())
                .all(|(l, r)| compare_values(l, r))
        }
        (Value::Number(l), Value::Number(r)) => (l - r).abs() < f64::EPSILON,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Null, Value::Null) => true,
        (Value::DateTime(l), Value::DateTime(r)) => l == r,
        _ => false,
    }
}

/// Asserts that two values are equal
///
/// # Panics
///
/// Panics if the values are not equal
pub fn assert_values_equal(left: &Value, right: &Value, message: &str) {
    assert!(
        compare_values(left, right),
        "{}\nLeft: {:?}\nRight: {:?}",
        message,
        left,
        right
    );
}
