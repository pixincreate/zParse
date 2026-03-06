use zparse::lexer::cursor::Cursor;

#[test]
fn test_cursor_basic() {
    let mut cursor = Cursor::new(b"hello");
    assert_eq!(cursor.current(), Some(b'h'));
    assert_eq!(cursor.peek(1), Some(b'e'));
    cursor.advance();
    assert_eq!(cursor.current(), Some(b'e'));
}

#[test]
fn test_cursor_whitespace() {
    let mut cursor = Cursor::new(b"  \t\nhello");
    cursor.skip_whitespace();
    assert_eq!(cursor.current(), Some(b'h'));
    assert_eq!(cursor.position().line, 2);
}

#[test]
fn test_cursor_consume() {
    let mut cursor = Cursor::new(b"abc");
    assert!(cursor.consume(b'a'));
    assert!(!cursor.consume(b'z'));
    assert_eq!(cursor.current(), Some(b'b'));
}

#[test]
fn test_cursor_eof() {
    let cursor = Cursor::new(b"");
    assert!(cursor.is_eof());
    assert_eq!(cursor.current(), None);
}

#[test]
fn test_cursor_slice() {
    let mut cursor = Cursor::new(b"hello world");
    let start = cursor.pos();
    cursor.advance();
    cursor.advance();
    cursor.advance();
    cursor.advance();
    assert_eq!(cursor.slice_from(start), b"hell");
}
