//! # zParse
//!
//! A Rust library for parsing JSON and TOML files with inter-conversion support.
//!
//! ## Features
//! - Parse JSON and TOML files
//! - Convert between JSON and TOML formats
//! - Formatting support
//! - Error handling with detailed messages
//!
//! ## Usage
//! ```rust
//! use zparse::{parse_file, Value};
//!
//! let value = parse_file("config.json")?;
//! // Work with parsed value
//! ```

use clap::Parser;
use std::path::Path;
use zparse::{
    converter::Converter,
    error::{ParseError, ParseErrorKind, Result},
    utils::{format_json, format_toml, parse_json, parse_toml, read_file, write_file},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file path
    #[arg(short, long)]
    file: String,

    /// Convert to format (json/toml)
    #[arg(short, long)]
    convert: Option<String>,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input file
    let content = read_file(&args.file)?;

    // Determine input format from file extension
    let input_ext = Path::new(&args.file)
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| {
            ParseError::new(ParseErrorKind::InvalidValue(
                "Invalid file extension".to_string(),
            ))
        })?;

    // Parse input
    let parsed_value = match input_ext.to_lowercase().as_str() {
        "json" => parse_json(&content)?,
        "toml" => parse_toml(&content)?,
        _ => {
            return Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Unsupported file format".to_string(),
            )))
        }
    };

    // Handle conversion if requested
    let (final_value, output_format) = if let Some(convert_to) = args.convert {
        match (input_ext, convert_to.as_str()) {
            ("json", "toml") => (Converter::json_to_toml(parsed_value)?, "toml"),
            ("toml", "json") => (Converter::toml_to_json(parsed_value)?, "json"),
            _ => {
                return Err(ParseError::new(ParseErrorKind::InvalidValue(
                    "Invalid conversion".to_string(),
                )))
            }
        }
    } else {
        (parsed_value, input_ext)
    };

    // Format the output
    let formatted_output = match output_format {
        "json" => format_json(&final_value),
        "toml" => format_toml(&final_value),
        _ => {
            return Err(ParseError::new(ParseErrorKind::InvalidValue(
                "Invalid output format".to_string(),
            )))
        }
    };

    // Write to file or print to stdout
    if let Some(output_path) = args.output {
        write_file(&output_path, &formatted_output)?;
    } else {
        println!("{}", formatted_output);
    }

    Ok(())
}
