//! DOM types for parsed JSON/TOML/YAML/XML values

use indexmap::map::{IntoIter, Iter, Keys, Values};
use indexmap::IndexMap;
use std::ops::Index;

/// A JSON/TOML/YAML/XML value
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// Null value
    #[default]
    Null,
    /// Boolean value
    Bool(bool),
    /// Numeric value (f64)
    Number(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Array),
    /// Object (key-value pairs with order preservation)
    Object(Object),
}

impl Value {
    /// Returns true if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns true if this value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns true if this value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns true if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns true if this value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns true if this value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Returns the boolean value if this is a boolean, None otherwise
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the numeric value if this is a number, None otherwise
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the string value if this is a string, None otherwise
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the array if this is an array, None otherwise
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the object if this is an object, None otherwise
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Returns a mutable reference to the array if this is an array, None otherwise
    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns a mutable reference to the object if this is an object, None otherwise
    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Number(value as f64)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Array> for Value {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(values: Vec<Value>) -> Self {
        Self::Array(Array(values))
    }
}

impl From<IndexMap<String, Value>> for Value {
    fn from(map: IndexMap<String, Value>) -> Self {
        Self::Object(Object(map))
    }
}

/// An order-preserving object (map of string keys to values)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Object(pub(crate) IndexMap<String, Value>);

impl Object {
    /// Creates a new empty object
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    /// Creates a new object with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(IndexMap::with_capacity(capacity))
    }

    /// Returns the number of key-value pairs in the object
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the object contains no key-value pairs
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the value corresponding to the key
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }

    /// Inserts a key-value pair into the object
    /// Returns the previous value if the key already existed
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> Option<Value> {
        self.0.insert(key.into(), value.into())
    }

    /// Removes a key from the object, returning the value if the key was present
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.swap_remove(key)
    }

    /// Returns true if the object contains the specified key
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Returns an iterator over the keys
    pub fn keys(&self) -> Keys<'_, String, Value> {
        self.0.keys()
    }

    /// Returns an iterator over the values
    pub fn values(&self) -> Values<'_, String, Value> {
        self.0.values()
    }

    /// Returns an iterator over key-value pairs
    pub fn iter(&self) -> Iter<'_, String, Value> {
        self.0.iter()
    }

    /// Returns an iterator that allows modifying each value
    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, String, Value> {
        self.0.iter_mut()
    }

    /// Clears the object, removing all key-value pairs
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl Index<&str> for Object {
    type Output = Value;

    #[allow(clippy::indexing_slicing)]
    fn index(&self, key: &str) -> &Self::Output {
        &self.0[key]
    }
}

impl Index<String> for Object {
    type Output = Value;

    #[allow(clippy::indexing_slicing)]
    fn index(&self, key: String) -> &Self::Output {
        &self.0[&key]
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a String, &'a Value);
    type IntoIter = Iter<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Object {
    type Item = (String, Value);
    type IntoIter = IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<IndexMap<String, Value>> for Object {
    fn from(map: IndexMap<String, Value>) -> Self {
        Self(map)
    }
}

impl FromIterator<(String, Value)> for Object {
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self(IndexMap::from_iter(iter))
    }
}

/// An array of values
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Array(pub(crate) Vec<Value>);

impl Array {
    /// Creates a new empty array
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new array with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Returns the number of elements in the array
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the array contains no elements
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the element at the given index
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Returns a mutable reference to the element at the given index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.0.get_mut(index)
    }

    /// Appends an element to the end of the array
    pub fn push(&mut self, value: impl Into<Value>) {
        self.0.push(value.into());
    }

    /// Removes the last element from the array and returns it
    pub fn pop(&mut self) -> Option<Value> {
        self.0.pop()
    }

    /// Returns an iterator over the array
    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.0.iter()
    }

    /// Returns an iterator that allows modifying each value
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Value> {
        self.0.iter_mut()
    }

    /// Clears the array, removing all values
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Inserts an element at the given index
    pub fn insert(&mut self, index: usize, value: impl Into<Value>) {
        self.0.insert(index, value.into());
    }

    /// Removes and returns the element at the given index
    pub fn remove(&mut self, index: usize) -> Value {
        self.0.remove(index)
    }
}

impl Index<usize> for Array {
    type Output = Value;

    #[allow(clippy::indexing_slicing)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Array {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<Value>> for Array {
    fn from(values: Vec<Value>) -> Self {
        Self(values)
    }
}

impl FromIterator<Value> for Array {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl IntoIterator for Value {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Array(arr) => arr.into_iter(),
            _ => Vec::new().into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(obj["name"], Value::String("Alice".to_string()));
        assert_eq!(obj["age"], Value::Number(30.0));

        let key = "name".to_string();
        assert_eq!(obj[key], Value::String("Alice".to_string()));
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

        assert_eq!(arr[0], Value::String("hello".to_string()));
        assert_eq!(arr[1], Value::Number(42.0));
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
        let arr = Array(vec![Value::Null, Value::Bool(true)]);
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
}
