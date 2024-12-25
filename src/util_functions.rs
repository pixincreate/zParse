use crate::enums::{ContentError, FileError, FileType, SkillIssue};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Error as SerdeJsonError, Value as SerdeJsonValue};
use std::{
    error::Error,
    process, {fs, io},
};
use strum::IntoEnumIterator;
use toml::{de::Error as TomlError, Value as TomlValue};

#[derive(Parser, Serialize, Deserialize)]
pub struct Args {
    /// The name of the file to read (path inclusive)
    #[arg(short, long)]
    pub file_name: String,
    /// The file type to convert to
    #[arg(short, long)]
    #[serde(alias = "convert_to")]
    pub convert: Option<FileType>,
    /// The output file name
    #[arg(short, long)]
    #[serde(alias = "out")]
    pub output: Option<String>,
}

pub struct ReturnArgs {
    pub file_type: FileType,
    pub contents: String,
    pub output: Option<String>,
}

pub fn read_file(filename: &str) -> Result<String, FileError> {
    fs::read_to_string(filename).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FileError::NotFound,
        _ => FileError::Io(e),
    })
}

/// Reads and validates a file of the specified type.
///
/// # Arguments
/// * `args` - Command line arguments vector containing file path
///
/// # Returns
/// * `Ok((FileType, String))` - The file type and its contents
/// * `Err(Box<dyn Error>)` - Any error that occurred during reading or parsing
pub fn read(args: Args) -> Result<ReturnArgs, Box<dyn Error>> {
    // args will be 3 if the command is correct. example: /targer/debug/zparse json file.json
    // if args.len() != 2 {
    if args.file_name.is_empty() {
        return Err(Box::new(SkillIssue::WrongCommand));
    }

    // Validate conversion type if specified
    if let Some(to_file_type) = args.convert.clone() {
        if !matches!(to_file_type, FileType::Json | FileType::Toml) {
            return Err(FileError::WrongExtension {
                expected: "json or toml".to_string(),
                found: format!("{:?}", to_file_type),
                supported: "json, toml".to_string(),
            }
            .into());
        }
    }

    if let Some(output) = args.output.clone() {
        if !output.is_empty() {
            let output_extension = output
                .split('.')
                .last()
                .ok_or(Box::new(FileError::InvalidExtension))?;

            match output_extension.to_lowercase().as_str() {
                "json" => FileType::Json,
                "toml" => FileType::Toml,
                _ => {
                    eprintln!(
                        "Invalid output file type: {}\nSupported types: {}",
                        output_extension,
                        FileType::iter()
                            .map(|x| format!("{:?}", x).to_lowercase())
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                    process::exit(1);
                }
            };
        }
    }

    let file_name = &args.file_name;
    let file_extension = file_name
        .split('.')
        .last()
        .ok_or(Box::new(FileError::InvalidExtension))?;

    let file_type = match file_extension.to_lowercase().as_str() {
        "json" => FileType::Json,
        "toml" => FileType::Toml,
        _ => {
            eprintln!(
                "Invalid file type: {}\nSupported types: {}",
                file_extension,
                FileType::iter()
                    .map(|x| format!("{:?}", x).to_lowercase())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            process::exit(1);
        }
    };

    let metadata = fs::metadata(file_name).map_err(|e| Box::new(FileError::Io(e)))?;

    if !metadata.is_file() {
        return Err(Box::new(FileError::NotFile));
    }

    let contents = read_file(file_name)?;

    match file_type {
        FileType::Json => match parse_json(&contents) {
            Ok(json_value) => {
                if let Some(FileType::Toml) = args.convert {
                    // Convert JSON to TOML
                    let toml_value: toml::Value = serde_json::from_value(json_value)
                        .map_err(|e| Box::new(ContentError::InvalidJson(e)))?;
                    Ok(pretty_print(
                        FileType::Toml,
                        toml::to_string_pretty(&toml_value).unwrap_or_default(),
                        args.output,
                    ))
                } else {
                    Ok(pretty_print(
                        FileType::Json,
                        json_value.to_string(),
                        args.output,
                    ))
                }
            }
            Err(error) => Err(Box::new(ContentError::InvalidJson(error))),
        },
        FileType::Toml => match parse_toml(&contents) {
            Ok(toml_value) => {
                if let Some(FileType::Json) = args.convert {
                    // Convert TOML to JSON
                    let json_value = serde_json::to_value(&toml_value)
                        .map_err(|e| Box::new(ContentError::InvalidJson(e)))?;
                    Ok(pretty_print(
                        FileType::Json,
                        serde_json::to_string_pretty(&json_value).unwrap_or_default(),
                        args.output,
                    ))
                } else {
                    Ok(pretty_print(
                        FileType::Toml,
                        toml_value.to_string(),
                        args.output,
                    ))
                }
            }
            Err(error) => Err(Box::new(ContentError::InvalidToml(error))),
        },
    }
}

fn parse_json(contents: &str) -> Result<SerdeJsonValue, SerdeJsonError> {
    serde_json::from_str(contents)
}

fn parse_toml(contents: &str) -> Result<TomlValue, TomlError> {
    toml::from_str(contents)
}

fn pretty_print(file_type: FileType, content: String, output: Option<String>) -> ReturnArgs {
    match file_type {
        FileType::Json => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                ReturnArgs {
                    file_type,
                    contents: serde_json::to_string_pretty(&json).unwrap_or(content),
                    output,
                }
            } else {
                ReturnArgs {
                    file_type,
                    contents: content,
                    output,
                }
            }
        }
        FileType::Toml => {
            if let Ok(toml) = content.parse::<TomlValue>() {
                ReturnArgs {
                    file_type,
                    contents: toml::to_string_pretty(&toml).unwrap_or(content),
                    output,
                }
            } else {
                ReturnArgs {
                    file_type,
                    contents: content,
                    output,
                }
            }
        }
    }
}
