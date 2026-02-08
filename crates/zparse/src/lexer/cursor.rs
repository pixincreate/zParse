//! Byte cursor for efficient input navigation

use crate::error::Pos;

/// Cursor for navigating byte input with position tracking
#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    input: &'a [u8],
    pos: usize,
    line: u32,
    col: u32,
}

impl<'a> Cursor<'a> {
    /// Create cursor from byte slice
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Get current byte without consuming
    pub const fn current(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    /// Peek at byte ahead without consuming
    pub const fn peek(&self, ahead: usize) -> Option<u8> {
        let idx = self.pos.saturating_add(ahead);
        if idx < self.input.len() {
            Some(self.input[idx])
        } else {
            None
        }
    }

    /// Advance cursor by one byte
    pub fn advance(&mut self) {
        if let Some(b) = self.current() {
            self.pos += 1;
            if b == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    /// Skip whitespace
    pub fn skip_whitespace(&mut self) {
        while let Some(b) = self.current() {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Consume byte if it matches
    pub fn consume(&mut self, expected: u8) -> bool {
        if self.current() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Get current position
    pub const fn position(&self) -> Pos {
        Pos::new(self.pos, self.line, self.col)
    }

    /// Check if at end of input
    pub const fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> &[u8] {
        &self.input[self.pos..]
    }

    /// Get current position index
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Get slice from start to current position
    pub fn slice_from(&self, start: usize) -> &'a [u8] {
        &self.input[start..self.pos]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(cursor.slice_from(start), b"hel");
    }
}
