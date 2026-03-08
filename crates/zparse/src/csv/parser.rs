use crate::csv::infer_primitive_value;
use crate::error::{Error, ErrorKind, Result, Span};
use crate::value::{Array, Object, Value};

pub const DEFAULT_DELIMITER: u8 = b',';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub delimiter: u8,
    pub max_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            delimiter: DEFAULT_DELIMITER,
            max_size: 0,
        }
    }
}

impl Config {
    pub const fn new(delimiter: u8, max_size: usize) -> Self {
        Self {
            delimiter,
            max_size,
        }
    }

    pub const fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub const fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }
}

#[derive(Clone, Debug)]
struct Field {
    value: String,
    quoted: bool,
}

#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a [u8],
    config: Config,
    bytes_parsed: usize,
}

impl<'a> Parser<'a> {
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            config: Config {
                delimiter: DEFAULT_DELIMITER,
                max_size: 0,
            },
            bytes_parsed: 0,
        }
    }

    pub fn with_delimiter(input: &'a [u8], delimiter: u8) -> Self {
        Self::with_config(input, Config::default().with_delimiter(delimiter))
    }

    pub fn with_config(input: &'a [u8], config: Config) -> Self {
        Self {
            input,
            config,
            bytes_parsed: 0,
        }
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub const fn bytes_parsed(&self) -> usize {
        self.bytes_parsed
    }

    pub fn parse(&mut self) -> Result<Value> {
        if matches!(self.config.delimiter, b'\n' | b'\r' | b'"') {
            return Err(Error::with_message(
                ErrorKind::InvalidToken,
                Span::empty(),
                "invalid CSV delimiter: delimiter cannot be newline, carriage return, or quote"
                    .to_string(),
            ));
        }

        if self.config.max_size > 0 && self.input.len() > self.config.max_size {
            return Err(Error::at(
                ErrorKind::MaxSizeExceeded {
                    max: self.config.max_size,
                },
                self.bytes_parsed,
                1,
                1,
            ));
        }

        let records = self.parse_records()?;
        self.bytes_parsed = self.input.len();
        if records.is_empty() {
            return Ok(Value::Array(Array::new()));
        }

        let headers = normalize_headers(
            records
                .first()
                .ok_or_else(|| invalid_csv("csv has no header row"))?,
        );

        let rows: Array = records
            .into_iter()
            .skip(1)
            .filter(|r| !is_blank_record(r))
            .map(|record| {
                let mut base: Object = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        let v = record.get(i).map(infer_field_value).unwrap_or(Value::Null);
                        (h.clone(), v)
                    })
                    .collect();

                let overflow: Object = record
                    .iter()
                    .enumerate()
                    .skip(headers.len())
                    .map(|(i, f)| (format!("column_{}", i + 1), infer_field_value(f)))
                    .collect();

                overflow.into_iter().for_each(|(k, v)| {
                    base.insert(k, v);
                });

                Value::Object(base)
            })
            .collect();

        Ok(Value::Array(rows))
    }

    fn parse_records(&self) -> Result<Vec<Vec<Field>>> {
        let mut records = Vec::new();
        let mut index = 0usize;

        while index < self.input.len() {
            if self.input.get(index).is_some_and(|byte| *byte == b'\n') {
                index += 1;
                continue;
            }

            if self.input.get(index).is_some_and(|byte| *byte == b'\r') {
                index += 1;
                if self.input.get(index).is_some_and(|byte| *byte == b'\n') {
                    index += 1;
                }
                continue;
            }

            let (record, next) = self.parse_record(index)?;
            if !is_blank_record(&record) {
                records.push(record);
            }
            index = next;
        }

        Ok(records)
    }

    fn parse_record(&self, mut index: usize) -> Result<(Vec<Field>, usize)> {
        let mut fields = Vec::new();

        loop {
            let (field, next) = self.parse_field(index)?;
            fields.push(field);
            index = next;

            if index >= self.input.len() {
                break;
            }

            match self.input.get(index).copied() {
                Some(delimiter) if delimiter == self.config.delimiter => {
                    index += 1;
                }
                Some(b'\n') => {
                    index += 1;
                    break;
                }
                Some(b'\r') => {
                    index += 1;
                    if self.input.get(index).is_some_and(|byte| *byte == b'\n') {
                        index += 1;
                    }
                    break;
                }
                _ => return Err(invalid_csv("invalid character after CSV field")),
            }
        }

        Ok((fields, index))
    }

    fn parse_field(&self, mut index: usize) -> Result<(Field, usize)> {
        if index >= self.input.len()
            || self
                .input
                .get(index)
                .is_some_and(|byte| *byte == self.config.delimiter)
            || self.input.get(index).is_some_and(|byte| *byte == b'\n')
            || self.input.get(index).is_some_and(|byte| *byte == b'\r')
        {
            return Ok((
                Field {
                    value: String::new(),
                    quoted: false,
                },
                index,
            ));
        }

        if self.input.get(index).is_some_and(|byte| *byte == b'"') {
            index += 1;
            let mut bytes = Vec::new();

            while index < self.input.len() {
                let byte = self
                    .input
                    .get(index)
                    .copied()
                    .ok_or_else(|| invalid_csv("unexpected end of csv field"))?;
                if byte == b'"' {
                    if self.input.get(index + 1).is_some_and(|next| *next == b'"') {
                        bytes.push(b'"');
                        index += 2;
                        continue;
                    }

                    index += 1;

                    while index < self.input.len()
                        && self
                            .input
                            .get(index)
                            .is_some_and(|byte| matches!(*byte, b' ' | b'\t' | b'\x0c'))
                    {
                        index += 1;
                    }

                    if index < self.input.len()
                        && self.input.get(index).is_some_and(|byte| {
                            *byte != self.config.delimiter && !matches!(*byte, b'\n' | b'\r')
                        })
                    {
                        return Err(invalid_csv("invalid character after quoted CSV field"));
                    }

                    let value =
                        String::from_utf8(bytes).map_err(|_| invalid_csv("csv must be utf-8"))?;

                    return Ok((
                        Field {
                            value,
                            quoted: true,
                        },
                        index,
                    ));
                }

                bytes.push(byte);
                index += 1;
            }

            return Err(invalid_csv("unterminated quoted CSV field"));
        }

        let start = index;
        while index < self.input.len()
            && self.input.get(index).is_some_and(|byte| {
                *byte != self.config.delimiter && !matches!(*byte, b'\n' | b'\r')
            })
        {
            index += 1;
        }

        let slice = self
            .input
            .get(start..index)
            .ok_or_else(|| invalid_csv("invalid csv span"))?;

        let raw = std::str::from_utf8(slice).map_err(|_| invalid_csv("csv must be utf-8"))?;

        Ok((
            Field {
                value: raw.to_string(),
                quoted: false,
            },
            index,
        ))
    }
}

fn normalize_headers(headers: &[Field]) -> Vec<String> {
    let mut names = Vec::with_capacity(headers.len());
    for (i, header) in headers.iter().enumerate() {
        let name = if header.quoted {
            header.value.clone()
        } else {
            header.value.trim().to_string()
        };

        let name = if name.is_empty() {
            format!("column_{}", i + 1)
        } else {
            name
        };

        let unique_name = (1..)
            .map(|s| {
                if s == 1 {
                    name.clone()
                } else {
                    format!("{name}_{s}")
                }
            })
            .find(|n| !names.contains(n))
            .unwrap_or(name);

        names.push(unique_name);
    }
    names
}

fn infer_field_value(field: &Field) -> Value {
    if field.quoted {
        return Value::String(field.value.clone());
    }

    let trimmed = field.value.trim();
    infer_primitive_value(trimmed).unwrap_or_else(|| Value::String(trimmed.to_string()))
}

fn is_blank_record(record: &[Field]) -> bool {
    record
        .iter()
        .all(|field| !field.quoted && field.value.trim().is_empty())
}

fn invalid_csv(message: &str) -> Error {
    Error::with_message(ErrorKind::InvalidToken, Span::empty(), message.to_string())
}
