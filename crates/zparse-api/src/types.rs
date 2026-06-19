use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InputFormat {
    Json,
    Jsonc,
    Csv,
    Toml,
    Yaml,
    Xml,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Csv,
    Toml,
    Yaml,
    Xml,
}

impl From<InputFormat> for zparse::Format {
    fn from(value: InputFormat) -> Self {
        match value {
            InputFormat::Json => zparse::Format::Json,
            InputFormat::Jsonc => zparse::Format::Json,
            InputFormat::Csv => zparse::Format::Csv,
            InputFormat::Toml => zparse::Format::Toml,
            InputFormat::Yaml => zparse::Format::Yaml,
            InputFormat::Xml => zparse::Format::Xml,
        }
    }
}

impl From<OutputFormat> for zparse::Format {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Json => zparse::Format::Json,
            OutputFormat::Csv => zparse::Format::Csv,
            OutputFormat::Toml => zparse::Format::Toml,
            OutputFormat::Yaml => zparse::Format::Yaml,
            OutputFormat::Xml => zparse::Format::Xml,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    pub content: String,
    pub format: InputFormat,
    pub csv_delimiter: Option<char>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub content: String,
    pub from: InputFormat,
    pub to: OutputFormat,
    pub csv_delimiter: Option<char>,
}

#[derive(Debug, Serialize)]
pub struct ApiResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}
