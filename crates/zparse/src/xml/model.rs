//! XML data model

use indexmap::IndexMap;

/// XML document
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub root: Element,
}

/// XML element
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: IndexMap<String, String>,
    pub children: Vec<Content>,
}

/// XML content node
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Element(Element),
    Text(String),
}
