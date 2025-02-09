use crate::error::{Location, ParseErrorKind, Result, SemanticError};
use crate::parser::Value;
use std::collections::HashMap;

pub struct ConversionContext {
    line: usize,
    column: usize,
    path: Vec<(String, usize, usize)>, // (key, line, column)
}

impl Default for ConversionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionContext {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            path: Vec::new(),
        }
    }

    pub fn create_location(&self) -> Location {
        Location::new(self.line, self.column)
    }

    pub fn enter_key(&mut self, key: &str, line: usize, column: usize) {
        self.line = line;
        self.column = column;
        self.path.push((key.to_string(), line, column));
    }

    pub fn exit_key(&mut self) {
        if let Some((_, line, column)) = self.path.pop() {
            self.line = line;
            self.column = column;
        }
    }

    pub fn get_path(&self) -> String {
        self.path
            .iter()
            .map(|(key, _, _)| key.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn update_position(&mut self, line: usize, column: usize) {
        self.line = line;
        self.column = column;
    }
}

pub trait CommonConverter {
    fn convert_map(map: HashMap<String, Value>, ctx: &mut ConversionContext) -> Result<Value>;
    fn convert_array(arr: Vec<Value>, ctx: &mut ConversionContext) -> Result<Value>;
    fn convert_value(value: Value, ctx: &mut ConversionContext) -> Result<Value>;

    fn validate_root(value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Map(map) => Ok(map),
            _ => {
                let location = Location::new(1, 1); // Root always starts at 1, 1
                Err(location.create_error(
                    ParseErrorKind::Semantic(SemanticError::TypeMismatch(
                        "Root must be an object/table".to_string(),
                    )),
                    "Invalid root value type",
                ))
            }
        }
    }

    fn convert_map_inner(
        map: HashMap<String, Value>,
        ctx: &mut ConversionContext,
    ) -> Result<HashMap<String, Value>> {
        let mut new_map = HashMap::new();
        for (key, value) in map {
            // Use key length to estimate column position
            let column = ctx.column + key.len() + 2; // +2 for quotes
            ctx.enter_key(&key, ctx.line, column);
            let converted = Self::convert_value(value, ctx).map_err(|e| {
                let location = ctx.create_location();
                let path = ctx.get_path();
                location.create_error(e.kind().clone(), &format!("Error at '{}': {}", path, e))
            })?;
            new_map.insert(key, converted);
            ctx.exit_key();
        }
        Ok(new_map)
    }

    fn convert_array_inner(arr: Vec<Value>, ctx: &mut ConversionContext) -> Result<Vec<Value>> {
        let mut converted_array = Vec::new();
        for (i, value) in arr.iter().enumerate() {
            // Estimate position based on index
            let column = ctx.column + i * 2 + 1; // Simple estimation
            ctx.enter_key(&i.to_string(), ctx.line, column);
            converted_array.push(Self::convert_value(value.clone(), ctx)?);
            ctx.exit_key();
        }

        Ok(converted_array)
    }
}
