use tracing::{debug, error, info, instrument, warn};

pub mod converter;
pub mod enums;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod utils;

pub use converter::Converter;
pub use error::{ParseError, ParseErrorKind, Result};
pub use parser::{json::JsonParser, toml::TomlParser, value::Value};
use utils::{parse_json, parse_toml};

#[instrument]
pub fn parse_file(path: &str) -> Result<Value> {
    debug!("Starting to parse file: {}", path);

    let content = std::fs::read_to_string(path).map_err(|e| {
        error!("Failed to read file: {}", e);
        ParseError::new(ParseErrorKind::IoError(e.to_string()))
    })?;

    info!("File read successfully, determining format");

    let result = if path.ends_with(".json") {
        parse_json(&content)
    } else if path.ends_with(".toml") {
        parse_toml(&content)
    } else {
        warn!("Unknown file extension");
        Err(ParseError::new(ParseErrorKind::UnknownFormat))
    };

    debug!("Parsing completed");
    result
}
