use std::collections::{hash_map::Iter, HashMap};

#[derive(Clone, Debug)]
pub struct Scope {
    name: String,
    fields: HashMap<&'static str, String>,
}

impl Scope {
    pub fn new(name: String, fields: HashMap<&'static str, String>) -> Self {
        Self { name, fields }
    }

    pub fn extend(&mut self, other: Self) {
        self.fields.extend(other.fields);
    }

    pub fn fields(&self) -> Iter<'_, &str, String> {
        self.fields.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
