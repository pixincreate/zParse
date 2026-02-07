//! Input abstraction for different sources

/// Input source abstraction
#[derive(Clone, Debug)]
pub struct Input<'a> {
    source: &'a [u8],
    filename: Option<&'a str>,
}

impl<'a> Input<'a> {
    /// Create from byte slice
    pub const fn from_bytes(source: &'a [u8]) -> Self {
        Self {
            source,
            filename: None,
        }
    }

    /// Create from string
    pub const fn from_str(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            filename: None,
        }
    }

    /// Set filename for error reporting
    pub const fn with_filename(mut self, filename: &'a str) -> Self {
        self.filename = Some(filename);
        self
    }

    /// Get source bytes
    pub const fn as_bytes(&self) -> &[u8] {
        self.source
    }

    /// Get filename if set
    pub const fn filename(&self) -> Option<&str> {
        self.filename
    }

    /// Get length in bytes
    pub const fn len(&self) -> usize {
        self.source.len()
    }

    /// Check if empty
    pub const fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(s: &'a str) -> Self {
        Self::from_str(s)
    }
}

impl<'a> From<&'a [u8]> for Input<'a> {
    fn from(b: &'a [u8]) -> Self {
        Self::from_bytes(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
