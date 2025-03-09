mod comparison;
mod data;
mod helpers;

pub use comparison::{assert_values_equal, compare_values};
pub use data::TestData;
pub use helpers::tmp_file_path;

// Re-export common test types/traits
pub use crate::{
    converter::Converter,
    error::{
        ConversionError, FormatError, LexicalError, ParseError, ParseErrorKind, Result,
        SecurityError, SemanticError, SyntaxError,
    },
    formatter::{FormatConfig, Formatter, JsonFormatter},
    parse_file,
    parser::{
        config::{ParserConfig, DEFAULT_MAX_DEPTH, DEFAULT_MAX_OBJECT_ENTRIES, DEFAULT_MAX_SIZE},
        json::JsonParser,
        toml::TomlParser,
        value::{values_equal, Value},
    },
    utils::{format_json, format_toml, parse_json, parse_toml, read_file, write_file},
};
