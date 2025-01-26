use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InternedString(Arc<str>);

pub struct StringInterner {
    strings: RwLock<HashMap<String, Arc<str>>>,
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: RwLock::new(HashMap::new()),
        }
    }

    pub fn intern(&self, s: &str) -> InternedString {
        if let Some(interned) = self.strings.read().get(s) {
            return InternedString(Arc::clone(interned));
        }

        let mut write_guard = self.strings.write();
        let interned = write_guard
            .entry(s.to_string())
            .or_insert_with(|| Arc::from(s))
            .clone();

        InternedString(interned)
    }
}
