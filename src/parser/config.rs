/// Configuration for parser limits and validation
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum nesting depth for objects/arrays
    pub max_depth: usize,
    /// Maximum input size in bytes
    pub max_size: usize,
    /// Maximum string length
    pub max_string_length: usize,
    /// Maximum number of object entries
    pub max_object_entries: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            max_size: 10 * 1024 * 1024, // 10MB
            max_string_length: 1000000,
            max_object_entries: 10000,
        }
    }
}
