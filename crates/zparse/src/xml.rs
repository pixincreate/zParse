//! XML parser module

pub mod model;
pub mod parser;

pub use model::{Content, Document, Element};
pub use parser::Parser;
