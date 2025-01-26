use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Json,
    Toml,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
        }
    }
}
