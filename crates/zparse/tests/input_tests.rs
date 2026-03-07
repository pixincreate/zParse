use zparse::Input;

#[test]
fn test_input_from_str() {
    let input = Input::from_str("hello");
    assert_eq!(input.len(), 5);
    assert!(!input.is_empty());
}

#[test]
fn test_input_with_filename() {
    let input = Input::from_str("test").with_filename("test.json");
    assert_eq!(input.filename(), Some("test.json"));
}

#[test]
fn test_empty_input() {
    let input = Input::from_str("");
    assert!(input.is_empty());
    assert_eq!(input.len(), 0);
}

#[test]
fn test_input_from_bytes() {
    let input: Input = b"bytes".as_slice().into();
    assert_eq!(input.len(), 5);
}

#[test]
fn test_input_from_str_trait() {
    let input: Input = "hello".into();
    assert_eq!(input.len(), 5);
}
