use zparse::{Array, Object, Value};

#[test]
fn test_value_is_methods() {
    assert!(Value::Null.is_null());
    assert!(!Value::Null.is_bool());
    assert!(!Value::Null.is_number());
    assert!(!Value::Null.is_string());
    assert!(!Value::Null.is_array());
    assert!(!Value::Null.is_object());

    assert!(Value::Bool(true).is_bool());
    assert!(Value::Number(42.0).is_number());
    assert!(Value::String("hello".to_string()).is_string());
    assert!(Value::Array(Array::new()).is_array());
    assert!(Value::Object(Object::new()).is_object());
}

#[test]
fn test_value_as_methods() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Bool(false).as_bool(), Some(false));
    assert_eq!(Value::Null.as_bool(), None);

    assert_eq!(Value::Number(42.0).as_number(), Some(42.0));
    assert_eq!(Value::Null.as_number(), None);

    assert_eq!(
        Value::String("hello".to_string()).as_string(),
        Some("hello")
    );
    assert_eq!(Value::Null.as_string(), None);

    assert!(Value::Array(Array::new()).as_array().is_some());
    assert_eq!(Value::Null.as_array(), None);

    assert!(Value::Object(Object::new()).as_object().is_some());
    assert_eq!(Value::Null.as_object(), None);
}

#[test]
fn test_value_from_impls() {
    let v: Value = true.into();
    assert!(matches!(v, Value::Bool(true)));

    let v: Value = 42.0.into();
    assert!(matches!(v, Value::Number(42.0)));

    let v: Value = 42i32.into();
    assert!(matches!(v, Value::Number(42.0)));

    let v: Value = "hello".into();
    assert!(matches!(v, Value::String(s) if s == "hello"));

    let v: Value = Array::new().into();
    assert!(matches!(v, Value::Array(_)));

    let v: Value = Object::new().into();
    assert!(matches!(v, Value::Object(_)));

    let v: Value = vec![Value::Null, Value::Bool(true)].into();
    assert!(matches!(v, Value::Array(arr) if arr.len() == 2));
}

#[test]
fn test_object_basics() {
    let mut obj = Object::new();
    assert!(obj.is_empty());
    assert_eq!(obj.len(), 0);

    obj.insert("key1", Value::String("value1".to_string()));
    assert!(!obj.is_empty());
    assert_eq!(obj.len(), 1);
    assert!(obj.contains_key("key1"));
    assert!(!obj.contains_key("key2"));

    assert_eq!(obj.get("key1"), Some(&Value::String("value1".to_string())));
    assert_eq!(obj.get("key2"), None);

    obj.insert("key2", 42i32);
    assert_eq!(obj.len(), 2);

    let removed = obj.remove("key1");
    assert!(removed.is_some());
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_object_index() {
    let mut obj = Object::new();
    obj.insert("name", "Alice");
    obj.insert("age", 30i32);

    assert_eq!(obj.get("name"), Some(&Value::String("Alice".to_string())));
    assert_eq!(obj.get("age"), Some(&Value::Number(30.0)));

    let key = "name".to_string();
    assert_eq!(obj.get(&key), Some(&Value::String("Alice".to_string())));
}

#[test]
fn test_object_order_preservation() {
    let mut obj = Object::new();
    obj.insert("first", 1i32);
    obj.insert("second", 2i32);
    obj.insert("third", 3i32);

    let keys: Vec<_> = obj.keys().collect();
    assert_eq!(keys, vec!["first", "second", "third"]);

    let values: Vec<_> = obj.values().collect();
    assert_eq!(
        values,
        vec![
            &Value::Number(1.0),
            &Value::Number(2.0),
            &Value::Number(3.0)
        ]
    );
}

#[test]
fn test_object_iter() {
    let mut obj = Object::new();
    obj.insert("a", 1i32);
    obj.insert("b", 2i32);

    let mut count = 0;
    for (k, v) in &obj {
        count += 1;
        assert!(matches!(v, Value::Number(1.0) | Value::Number(2.0)));
        assert!(k == "a" || k == "b");
    }
    assert_eq!(count, 2);

    let obj2: Object = obj.into_iter().collect();
    assert_eq!(obj2.len(), 2);
}

#[test]
fn test_array_basics() {
    let mut arr = Array::new();
    assert!(arr.is_empty());
    assert_eq!(arr.len(), 0);

    arr.push(Value::Null);
    assert!(!arr.is_empty());
    assert_eq!(arr.len(), 1);

    arr.push(42i32);
    assert_eq!(arr.len(), 2);

    assert_eq!(arr.get(0), Some(&Value::Null));
    assert_eq!(arr.get(1), Some(&Value::Number(42.0)));
    assert_eq!(arr.get(2), None);

    let popped = arr.pop();
    assert_eq!(popped, Some(Value::Number(42.0)));
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_array_index() {
    let mut arr = Array::new();
    arr.push("hello");
    arr.push(42i32);

    assert_eq!(arr.get(0), Some(&Value::String("hello".to_string())));
    assert_eq!(arr.get(1), Some(&Value::Number(42.0)));
}

#[test]
fn test_array_iter() {
    let mut arr = Array::new();
    arr.push(1i32);
    arr.push(2i32);
    arr.push(3i32);

    let mut sum = 0.0;
    for v in &arr {
        if let Value::Number(n) = v {
            sum += n;
        }
    }
    assert_eq!(sum, 6.0);

    let arr2: Array = arr.into_iter().collect();
    assert_eq!(arr2.len(), 3);
}

#[test]
fn test_value_into_iterator() {
    let mut arr = Array::new();
    arr.push(Value::Null);
    arr.push(Value::Bool(true));
    let value = Value::Array(arr);

    let collected: Vec<_> = value.into_iter().collect();
    assert_eq!(collected.len(), 2);
}

#[test]
fn test_non_array_value_into_iterator() {
    let value = Value::Null;
    let collected: Vec<_> = value.into_iter().collect();
    assert!(collected.is_empty());
}
