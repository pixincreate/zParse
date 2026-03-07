use zparse::{Error, ErrorKind, Pos};

#[test]
fn test_pos_display() {
    let pos = Pos::new(42, 10, 5);
    assert_eq!(pos.to_string(), "42:10:5");
}

#[test]
fn test_error_creation() {
    let err = Error::at(ErrorKind::InvalidToken, 0, 1, 1);
    assert_eq!(err.kind(), &ErrorKind::InvalidToken);
}

#[test]
fn test_error_display() {
    let err = Error::at(ErrorKind::InvalidEscapeSequence, 10, 2, 5);
    let display = err.to_string();
    assert!(display.contains("error at"));
    assert!(display.contains("invalid escape sequence"));
}
