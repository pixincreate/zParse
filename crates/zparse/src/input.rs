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
