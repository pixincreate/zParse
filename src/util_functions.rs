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

pub fn validate_file(file_type: FileType, filename: &str) {
    if let Ok(metadata) = fs::metadata(filename) {
        if metadata.is_file() {
            let file_extension = filename.split('.').last();

            match file_type {
                FileType::Json => {
                    if file_extension != Some("json") {
                        eprintln!(
                            "Invalid file type: {}\nSupport types: {}",
                            filename,
                            FileType::iter()
                                .map(|x| format!("{:?}", x).to_lowercase())
                                .collect::<Vec<String>>()
                                .join(", ")
                        );
                        std::process::exit(1);
                    }
                }
                FileType::Toml => {
                    if file_extension != Some("toml") {
                        eprintln!(
                            "Invalid file type: {}\nSupport types: {}",
                            filename,
                            FileType::iter()
                                .map(|x| format!("{:?}", x).to_lowercase())
                                .collect::<Vec<String>>()
                                .join(", ")
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

pub fn read(args: Vec<String>) -> Result<(FileType, String), Box<dyn Error>> {
    // args will be 3 if the command is correct. example: /targer/debug/zparse json file.json
    if args.len() != 2 {
        return Err(Box::new(SkillIssue::WrongCommand));
    }

    let file_extension = args[1].split('.').last().unwrap_or_default();

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
    let file_name = &args[1];

    validate_file(file_type.clone(), file_name);

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
        FileType::Json => (
            file_type,
            serde_json::to_string_pretty(&content).unwrap_or_default(),
        ),
        FileType::Toml => {
            // Parse the content as TOML first
            if let Ok(toml_value) = content.parse::<TomlValue>() {
                (
                    file_type,
                    toml::to_string_pretty(&toml_value).unwrap_or_default(),
                )
            } else {
                (file_type, content) // Return original content if parsing fails
            }
        }
    }
}
