//! JSON streaming parser module

pub mod event;
pub mod parser;

pub use event::Event;
pub use parser::{Config, Parser};
