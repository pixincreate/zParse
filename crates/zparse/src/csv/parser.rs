use crate::error::{Error, ErrorKind, Result, Span};
use crate::value::{Array, Object, Value};

#[derive(Clone, Debug)]
struct Field {
    value: String,
    quoted: bool,
}

#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a [u8],
}

impl<'a> Parser<'a> {
    pub const fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    pub fn parse(&mut self) -> Result<Value> {
        let records = self.parse_records()?;
        if records.is_empty() {
            return Ok(Value::Array(Array::new()));
        }

        let headers = normalize_headers(
            records
                .first()
                .ok_or_else(|| invalid_csv("csv has no header row"))?,
        );
        let mut rows = Array::new();

        for record in records.into_iter().skip(1) {
            if is_blank_record(&record) {
                continue;
            }

            let mut obj = Object::new();

            for (index, header) in headers.iter().enumerate() {
                let value = record
                    .get(index)
                    .map(infer_field_value)
                    .unwrap_or(Value::Null);
                obj.insert(header.clone(), value);
            }

            if record.len() > headers.len() {
                for (index, field) in record.iter().enumerate().skip(headers.len()) {
                    let header = format!("column_{}", index + 1);
                    obj.insert(header, infer_field_value(field));
                }
            }

            rows.push(Value::Object(obj));
        }

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
                Some(b',') => {
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
            || self.input.get(index).is_some_and(|byte| *byte == b',')
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
                        && self
                            .input
                            .get(index)
                            .is_some_and(|byte| !matches!(*byte, b',' | b'\n' | b'\r'))
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
            && self
                .input
                .get(index)
                .is_some_and(|byte| !matches!(*byte, b',' | b'\n' | b'\r'))
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

    for (index, header) in headers.iter().enumerate() {
        let mut name = if header.quoted {
            header.value.clone()
        } else {
            header.value.trim().to_string()
        };

        if name.is_empty() {
            name = format!("column_{}", index + 1);
        }

        let mut candidate = name.clone();
        let mut suffix = 2usize;
        while names.iter().any(|existing| existing == &candidate) {
            candidate = format!("{name}_{suffix}");
            suffix += 1;
        }
        names.push(candidate);
    }

    names
}

fn infer_field_value(field: &Field) -> Value {
    if !field.quoted {
        let trimmed = field.value.trim();
        if trimmed.is_empty() {
            return Value::Null;
        }

        if trimmed.eq_ignore_ascii_case("null") {
            return Value::Null;
        }

        if trimmed.eq_ignore_ascii_case("true") {
            return Value::Bool(true);
        }

        if trimmed.eq_ignore_ascii_case("false") {
            return Value::Bool(false);
        }

        if trimmed.parse::<i64>().is_ok()
            && let Ok(number) = trimmed.parse::<f64>()
            && number.is_finite()
        {
            return Value::Number(number);
        }

        if let Ok(float) = trimmed.parse::<f64>()
            && float.is_finite()
        {
            return Value::Number(float);
        }

        return Value::String(trimmed.to_string());
    }

    Value::String(field.value.clone())
}

fn is_blank_record(record: &[Field]) -> bool {
    record
        .iter()
        .all(|field| !field.quoted && field.value.trim().is_empty())
}

fn invalid_csv(message: &str) -> Error {
    Error::with_message(ErrorKind::InvalidToken, Span::empty(), message.to_string())
}
