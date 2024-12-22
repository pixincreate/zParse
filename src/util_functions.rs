use crate::enums::{ContentError, FileError, FileType, SkillIssue};
use serde_json::{Error as SerdeJsonError, Value as SerdeJsonValue};
use std::{
    error::Error,
    process, {fs, io},
};
use strum::IntoEnumIterator;
use toml::{de::Error as TomlError, Value as TomlValue};

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
pub fn read(args: Vec<String>) -> Result<(FileType, String), Box<dyn Error>> {
    // args will be 3 if the command is correct. example: /targer/debug/zparse json file.json
    if args.len() != 2 {
        return Err(Box::new(SkillIssue::WrongCommand));
    }

    let file_name = &args[1];
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

    match read_file(file_name) {
        Ok(contents) => match file_type {
            FileType::Json => match parse_json(&contents) {
                Ok(contents) => Ok(pretty_print(file_type, contents.to_string())),
                Err(error) => Err(Box::new(ContentError::InvalidJson(error))),
            },
            FileType::Toml => match parse_toml(&contents) {
                Ok(contents) => Ok(pretty_print(file_type, contents.to_string())),
                Err(error) => Err(Box::new(ContentError::InvalidToml(error))),
            },
        },
        Err(error) => Err(Box::new(error)),
    }
}

fn parse_json(contents: &str) -> Result<SerdeJsonValue, SerdeJsonError> {
    serde_json::from_str(contents)
}

fn parse_toml(contents: &str) -> Result<TomlValue, TomlError> {
    toml::from_str(contents)
}

fn pretty_print(file_type: FileType, content: String) -> (FileType, String) {
    match file_type {
        FileType::Json => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                (
                    file_type,
                    serde_json::to_string_pretty(&json).unwrap_or(content),
                )
            } else {
                (file_type, content)
            }
        }
        FileType::Toml => {
            if let Ok(toml) = content.parse::<TomlValue>() {
                (file_type, toml::to_string_pretty(&toml).unwrap_or(content))
            } else {
                (file_type, content)
            }
        }
    }
}
