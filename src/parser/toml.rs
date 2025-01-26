use super::{lexer::Lexer, token::Token, value::Value};
use crate::error::{ParseError, ParseErrorKind, Result};
use std::collections::HashMap;

pub struct TomlParser {
    lexer: Lexer,
    current_token: Token,
    tables: HashMap<String, Value>,
    current_table: Vec<String>,
}

impl TomlParser {
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
            tables: HashMap::new(),
            current_table: Vec::new(),
        })
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Value> {
        while self.current_token != Token::EOF {
            match self.current_token {
                Token::LeftBracket => {
                    self.parse_table_header()?;
                }
                Token::String(_) => {
                    self.parse_key_value()?;
                }
                Token::EOF => break,
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(format!(
                        "{:?}",
                        self.current_token
                    ))));
                }
            }
        }

        Ok(Value::Table(self.tables.clone()))
    }

    fn parse_table_header(&mut self) -> Result<()> {
        self.advance()?; // consume first '['
        let is_array_table = matches!(self.current_token, Token::LeftBracket);
        if is_array_table {
            self.advance()?; // consume second '['
        }

        let mut path = Vec::new();

        loop {
            match &self.current_token {
                Token::String(s) => {
                    path.push(s.clone());
                    self.advance()?;
                }
                Token::Dot => {
                    self.advance()?;
                    continue;
                }
                Token::RightBracket => {
                    self.advance()?;
                    if is_array_table {
                        if self.current_token != Token::RightBracket {
                            return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                                "Expected closing bracket for array table".to_string(),
                            )));
                        }
                        self.advance()?; // consume second ']'
                        self.current_table = path.clone();
                        self.get_or_create_array_table(&path)?;
                    } else {
                        self.current_table = path;
                        self.ensure_table_exists()?;
                    }
                    break;
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Invalid table header".to_string(),
                    )))
                }
            }
        }

        Ok(())
    }

    fn ensure_table_exists(&mut self) -> Result<()> {
        let mut current = &mut self.tables;
        let mut path = Vec::new();

        for key in &self.current_table {
            path.push(key);

            // Check if this path already exists as a complete table
            if path.len() == self.current_table.len() && current.contains_key(key) {
                return Err(ParseError::new(ParseErrorKind::InvalidValue(format!(
                    "Duplicate table definition: [{}]",
                    path.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(".")
                ))));
            }

            // Handle existing key
            match current.get(key) {
                Some(Value::Table(_)) => {
                    current = match current.get_mut(key) {
                        Some(Value::Table(table)) => table,
                        _ => return Err(ParseError::new(ParseErrorKind::NestedTableError)),
                    };
                }
                Some(_) => {
                    return Err(ParseError::new(ParseErrorKind::InvalidValue(format!(
                        "Key '{}' is already defined with a different type",
                        path.iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(".")
                    ))));
                }
                None => {
                    // Create new table
                    current.insert(key.clone(), Value::Table(HashMap::new()));
                    current = match current.get_mut(key) {
                        Some(Value::Table(table)) => table,
                        _ => return Err(ParseError::new(ParseErrorKind::NestedTableError)),
                    };
                }
            }
        }

        Ok(())
    }

    fn parse_key_value(&mut self) -> Result<()> {
        let key = match &self.current_token {
            Token::String(s) => s.clone(),
            _ => {
                return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                    "Expected key".to_string(),
                )))
            }
        };
        self.advance()?;

        if self.current_token != Token::Equals {
            return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                "Expected =".to_string(),
            )));
        }
        self.advance()?;

        let value = self.parse_value()?;

        // Get the current table
        let mut current = &mut self.tables;
        for table_key in &self.current_table {
            current = match current.get_mut(table_key) {
                Some(Value::Table(table)) => table,
                Some(Value::Array(arr)) => {
                    if let Some(Value::Table(table)) = arr.last_mut() {
                        table
                    } else {
                        return Err(ParseError::new(ParseErrorKind::NestedTableError));
                    }
                }
                _ => return Err(ParseError::new(ParseErrorKind::NestedTableError)),
            };
        }

        current.insert(key, value);
        Ok(())
    }

    fn parse_value(&mut self) -> Result<Value> {
        match self.current_token {
            Token::String(ref s) => {
                let value = Value::String(s.clone());
                self.advance()?;
                Ok(value)
            }
            Token::Number(n) => {
                let value = Value::Number(n);
                self.advance()?;
                Ok(value)
            }
            Token::Boolean(b) => {
                let value = Value::Boolean(b);
                self.advance()?;
                Ok(value)
            }
            Token::DateTime(ref dt) => {
                let value = Value::DateTime(dt.clone());
                self.advance()?;
                Ok(value)
            }
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_inline_table(),
            _ => Err(ParseError::new(ParseErrorKind::UnexpectedToken(format!(
                "{:?}",
                self.current_token
            )))),
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        let mut array = Vec::new();
        self.advance()?; // consume '['

        while self.current_token != Token::RightBracket {
            let value = self.parse_value()?;
            array.push(value);

            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                    if self.current_token == Token::RightBracket {
                        return Err(ParseError::new(ParseErrorKind::InvalidValue(
                            "Trailing comma".to_string(),
                        )));
                    }
                }
                Token::RightBracket => break,
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Expected , or ]".to_string(),
                    )))
                }
            }
        }

        self.advance()?; // consume ']'
        Ok(Value::Array(array))
    }

    fn parse_inline_table(&mut self) -> Result<Value> {
        let mut map = HashMap::new();
        self.advance()?; // consume '{'

        if self.current_token == Token::RightBrace {
            self.advance()?;
            return Ok(Value::Table(map));
        }

        loop {
            let key = match self.current_token {
                Token::String(ref s) => s.clone(),
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Expected key".to_string(),
                    )))
                }
            };
            self.advance()?;

            if self.current_token != Token::Equals {
                return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                    "Expected =".to_string(),
                )));
            }
            self.advance()?;

            let value = self.parse_value()?;
            map.insert(key, value);

            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                }
                Token::RightBrace => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken(
                        "Expected , or }".to_string(),
                    )))
                }
            }
        }

        Ok(Value::Table(map))
    }

    fn get_or_create_array_table(&mut self, path: &[String]) -> Result<()> {
        let mut current = &mut self.tables;
        let mut temp_path = Vec::new();

        for key in path {
            temp_path.push(key.clone());

            // Check if we need to create the key
            if !current.contains_key(key) {
                if temp_path.len() == path.len() {
                    // Last key - create an array
                    current.insert(
                        key.clone(),
                        Value::Array(vec![Value::Table(HashMap::new())]),
                    );
                } else {
                    // Intermediate key - create a table
                    current.insert(key.clone(), Value::Table(HashMap::new()));
                }
            } else {
                // Handle existing key
                if temp_path.len() == path.len() {
                    // Last key - append to array if it exists
                    match current.get_mut(key) {
                        Some(Value::Array(arr)) => {
                            arr.push(Value::Table(HashMap::new()));
                        }
                        _ => return Err(ParseError::new(ParseErrorKind::NestedTableError)),
                    }
                }
            }

            // Move to next level
            current = match current.get_mut(key) {
                Some(Value::Table(table)) => table,
                Some(Value::Array(arr)) => {
                    if let Some(Value::Table(table)) = arr.last_mut() {
                        table
                    } else {
                        return Err(ParseError::new(ParseErrorKind::NestedTableError));
                    }
                }
                _ => return Err(ParseError::new(ParseErrorKind::NestedTableError)),
            };
        }

        Ok(())
    }
}
