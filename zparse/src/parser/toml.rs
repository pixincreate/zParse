use super::{
    config::{ParserConfig, ParsingContext},
    lexer::Lexer,
    value::Value,
};
use crate::common::parser_state::ParserState;
use crate::enums::Token;
use crate::error::{
    LexicalError, ParseError, ParseErrorKind, Result, SecurityError, SemanticError, SyntaxError,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct TomlParser {
    /// Lexer that provides tokens
    lexer: Lexer,
    /// Current token being processed
    current_token: Token,
    /// Parsed tables
    tables: HashMap<String, Value>,
    /// Current table path
    current_table: Vec<String>,
    /// Parser for TOML documents
    config: ParserConfig,
    /// Parsing context for tracking depth and size
    context: ParsingContext,
}

impl TomlParser {
    pub fn new(input: &str) -> Result<Self> {
        let state = ParserState::new();

        // Check input size first
        state.validate_input_size(input.len())?;

        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;

        // Initialize with default config
        Ok(Self {
            lexer,
            current_token,
            tables: HashMap::new(),
            current_table: Vec::new(),
            config: ParserConfig::default(),
            context: ParsingContext::new(),
        })
    }

    /// Setter method to configure the parser
    pub fn with_config(mut self, config: ParserConfig) -> Self {
        self.config = config;
        self
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
                    return Err(ParseError::new(ParseErrorKind::Lexical(
                        LexicalError::UnexpectedToken(format!("{:?}", self.current_token)),
                    )));
                }
            }
        }

        Ok(Value::Map(self.tables.clone()))
    }

    fn parse_table_header(&mut self) -> Result<()> {
        // Check nesting depth before processing table
        self.context.enter_nested(&self.config)?;

        self.advance()?; // consume first '['
        let is_array_table = matches!(self.current_token, Token::LeftBracket);
        if is_array_table {
            self.advance()?; // consume second '['
        }

        let mut path = Vec::new();
        let mut entry_count = 0;

        // Check max depth before processing
        if self.context.current_depth >= self.config.max_depth {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxDepthExceeded,
            )));
        }

        loop {
            match &self.current_token {
                Token::String(s) => {
                    // Check max depth on each new component
                    if path.len() >= self.config.max_depth {
                        return Err(ParseError::new(ParseErrorKind::Security(
                            SecurityError::MaxDepthExceeded,
                        )));
                    }

                    // Validate string length and track memory
                    self.config.validate_string(s)?;
                    // Track memory usage
                    self.context.add_size(s.len(), &self.config)?;

                    entry_count += 1;
                    self.config.validate_object_entries(entry_count)?;

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
                            return Err(ParseError::new(ParseErrorKind::Lexical(
                                LexicalError::UnexpectedToken(format!(
                                    "{:?}. Expected ] for array table",
                                    self.current_token,
                                )),
                            )));
                        }
                        self.advance()?; // consume second ']'
                        self.detect_circular_reference(&path)?;
                        self.current_table = path.clone();
                        self.get_or_create_array_table(&path)?;
                    } else {
                        self.detect_circular_reference(&path)?;
                        self.current_table = path;
                        self.ensure_table_exists()?;
                    }
                    break;
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::Lexical(
                        LexicalError::UnexpectedToken(format!(
                            "{:?}. Invalid table header",
                            self.current_token
                        )),
                    )))
                }
            }
        }

        self.context.exit_nested();
        Ok(())
    }

    fn ensure_table_exists(&mut self) -> Result<()> {
        let mut current = &mut self.tables;
        let mut path = Vec::new();

        // Check total depth before processing
        if self.current_table.len() > self.config.max_depth {
            return Err(ParseError::new(ParseErrorKind::Security(
                SecurityError::MaxDepthExceeded,
            )));
        }

        for key in &self.current_table {
            // Track nesting level
            self.context.enter_nested(&self.config)?;
            path.push(key);

            // First check if the key exists and if it's a non-table type
            if let Some(existing) = current.get(key) {
                match existing {
                    Value::Map(_) => {
                        // If this is the final component and it already exists as a table
                        if path.len() == self.current_table.len() {
                            return Err(ParseError::new(ParseErrorKind::Syntax(
                                SyntaxError::InvalidValue(format!(
                                    "Duplicate table definition: [{}]",
                                    path.iter()
                                        .map(|s| s.as_str())
                                        .collect::<Vec<_>>()
                                        .join(".")
                                )),
                            )));
                        }
                    }
                    Value::Array(_) => {
                        return Err(ParseError::new(ParseErrorKind::Semantic(
                            SemanticError::NestedTableError,
                        )))
                    }
                    Value::Number(n) => {
                        return Err(ParseError::new(ParseErrorKind::Semantic(
                            SemanticError::TypeMismatch(format!(
                                "Expected table, found Number({})",
                                // Format number without decimal point if it's a whole number
                                if n.fract() == 0.0 {
                                    n.trunc().to_string()
                                } else {
                                    n.to_string()
                                }
                            )),
                        )));
                    }
                    other => {
                        return Err(ParseError::new(ParseErrorKind::Semantic(
                            SemanticError::TypeMismatch(format!(
                                "Expected table, found {:?}",
                                other
                            )),
                        )))
                    }
                }
            }

            // Handle existing or create new key
            match current.entry(key.clone()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(Value::Map(HashMap::new()));
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    match entry.get_mut() {
                        Value::Map(_) => {} // OK - continue traversing
                        Value::Array(_) => {
                            return Err(ParseError::new(ParseErrorKind::Semantic(
                                SemanticError::NestedTableError,
                            )))
                        }
                        other => {
                            return Err(ParseError::new(ParseErrorKind::Semantic(
                                SemanticError::TypeMismatch(format!(
                                    "Expected table, found {:?}",
                                    other
                                )),
                            )))
                        }
                    }
                }
            }

            current = match current.get_mut(key) {
                Some(Value::Map(table)) => table,
                _ => {
                    return Err(ParseError::new(ParseErrorKind::Semantic(
                        SemanticError::NestedTableError,
                    )))
                }
            };

            self.context.exit_nested();
        }

        Ok(())
    }

    fn parse_key_value(&mut self) -> Result<()> {
        // Track nesting level for complex values
        self.context.enter_nested(&self.config)?;

        let key = match &self.current_token {
            Token::String(s) => {
                // Validate string length
                self.config.validate_string(s)?;
                // Track memory usage
                self.context.add_size(s.len(), &self.config)?;

                s.clone()
            }
            _ => {
                return Err(ParseError::new(ParseErrorKind::Lexical(
                    LexicalError::UnexpectedToken(format!(
                        "{:?}. Expected key.",
                        self.current_token
                    )),
                )))
            }
        };
        self.advance()?;

        if self.current_token != Token::Equals {
            return Err(ParseError::new(ParseErrorKind::Lexical(
                LexicalError::UnexpectedToken(format!("{:?}. Expected =", self.current_token)),
            )));
        }
        self.advance()?;

        let value = self.parse_value()?;

        // Get the current table
        let mut current = &mut self.tables;
        for table_key in &self.current_table {
            current = match current.get_mut(table_key) {
                Some(Value::Map(table)) => table,
                Some(Value::Array(arr)) => {
                    if let Some(Value::Map(table)) = arr.last_mut() {
                        table
                    } else {
                        return Err(ParseError::new(ParseErrorKind::Semantic(
                            SemanticError::NestedTableError,
                        )));
                    }
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::Semantic(
                        SemanticError::NestedTableError,
                    )))
                }
            };
        }

        current.insert(key, value);
        self.context.exit_nested();
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
            _ => Err(ParseError::new(ParseErrorKind::Lexical(
                LexicalError::UnexpectedToken(format!("{:?}", self.current_token)),
            ))),
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.context.enter_nested(&self.config)?;

        let mut array = Vec::new();
        self.advance()?; // consume '['

        let mut entry_count = 0;

        while self.current_token != Token::RightBracket {
            entry_count += 1;
            self.config.validate_object_entries(entry_count)?;

            let value = self.parse_value()?;
            array.push(value);

            match self.current_token {
                Token::Comma => {
                    self.advance()?;
                    if self.current_token == Token::RightBracket {
                        return Err(ParseError::new(ParseErrorKind::Syntax(
                            SyntaxError::InvalidValue(format!(
                                "Trailing comma in {:?}",
                                self.current_token
                            )),
                        )));
                    }
                }
                Token::RightBracket => break,
                _ => {
                    return Err(ParseError::new(ParseErrorKind::Lexical(
                        LexicalError::UnexpectedToken(format!(
                            "{:?}. Expected , or ]",
                            self.current_token
                        )),
                    )))
                }
            }
        }

        self.advance()?; // consume ']'
        self.context.exit_nested();
        Ok(Value::Array(array))
    }

    fn parse_inline_table(&mut self) -> Result<Value> {
        self.context.enter_nested(&self.config)?;

        let mut map = HashMap::new();
        self.advance()?; // consume '{'

        let mut entry_count = 0;

        if self.current_token == Token::RightBrace {
            self.advance()?;
            self.context.exit_nested();
            return Ok(Value::Map(map));
        }

        loop {
            entry_count += 1;
            self.config.validate_object_entries(entry_count)?;

            let key = match &self.current_token {
                Token::String(s) => {
                    // Validate string length
                    self.config.validate_string(s)?;
                    // Track memory usage
                    self.context.add_size(s.len(), &self.config)?;

                    s.clone()
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::Lexical(
                        LexicalError::UnexpectedToken(format!(
                            "{:?}. Expected key.",
                            self.current_token
                        )),
                    )))
                }
            };
            self.advance()?;

            if self.current_token != Token::Equals {
                return Err(ParseError::new(ParseErrorKind::Lexical(
                    LexicalError::UnexpectedToken(format!("{:?}. Expected =", self.current_token)),
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
                    return Err(ParseError::new(ParseErrorKind::Lexical(
                        LexicalError::UnexpectedToken(format!(
                            "{:?}. Expected , or }}",
                            self.current_token
                        )),
                    )))
                }
            }
        }

        self.context.exit_nested();
        Ok(Value::Map(map))
    }

    fn get_or_create_array_table(&mut self, path: &[String]) -> Result<()> {
        // Check nesting depth
        self.context.enter_nested(&self.config)?;

        let mut current = &mut self.tables;
        let mut temp_path = Vec::new();

        for key in path {
            // Check depth for each nested level
            self.context.enter_nested(&self.config)?;
            temp_path.push(key.clone());

            // Check if we need to create the key
            if !current.contains_key(key) {
                if temp_path.len() == path.len() {
                    // Last key - create an array
                    current.insert(key.clone(), Value::Array(vec![Value::Map(HashMap::new())]));
                } else {
                    // Intermediate key - create a table
                    current.insert(key.clone(), Value::Map(HashMap::new()));
                }
            } else {
                // Handle existing key
                if temp_path.len() == path.len() {
                    // Last key - append to array if it exists
                    match current.get_mut(key) {
                        Some(Value::Array(arr)) => {
                            // Validate array size
                            self.config.validate_object_entries(arr.len() + 1)?;
                            arr.push(Value::Map(HashMap::new()));
                        }
                        _ => {
                            return Err(ParseError::new(ParseErrorKind::Semantic(
                                SemanticError::NestedTableError,
                            )))
                        }
                    }
                }
            }

            // Move to next level
            current = match current.get_mut(key) {
                Some(Value::Map(table)) => table,
                Some(Value::Array(arr)) => {
                    if let Some(Value::Map(table)) = arr.last_mut() {
                        table
                    } else {
                        return Err(ParseError::new(ParseErrorKind::Semantic(
                            SemanticError::NestedTableError,
                        )));
                    }
                }
                _ => {
                    return Err(ParseError::new(ParseErrorKind::Semantic(
                        SemanticError::NestedTableError,
                    )))
                }
            };

            self.context.exit_nested();
        }

        self.context.exit_nested();
        Ok(())
    }

    fn detect_circular_reference(&self, path: &[String]) -> Result<()> {
        let full_path = path.join(".");
        let mut current_path = String::new();

        for segment in path {
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(segment);

            // Check if this path exists and references the target path
            if let Some(Value::Map(table)) = self.tables.get(&current_path) {
                if contains_path(table, &full_path) {
                    return Err(ParseError::new(ParseErrorKind::Semantic(
                        SemanticError::CircularReference,
                    )));
                }
            }
        }
        Ok(())
    }
}

fn contains_path(table: &HashMap<String, Value>, target_path: &str) -> bool {
    for (key, value) in table {
        match value {
            Value::Map(inner_table) => {
                let current_path = format!("{}.{}", target_path, key);
                if current_path == target_path || contains_path(inner_table, target_path) {
                    return true;
                }
            }
            _ => continue,
        }
    }
    false
}
