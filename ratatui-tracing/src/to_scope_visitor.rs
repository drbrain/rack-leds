use std::collections::HashMap;

use tracing::field::{Field, Visit};

use crate::Scope;

#[derive(Default)]
pub struct ToScopeVisitor {
    fields: HashMap<&'static str, String>,
}

impl ToScopeVisitor {
    pub fn finish(self, name: String) -> Scope {
        Scope::new(name, self.fields)
    }

    pub fn fields(self) -> HashMap<&'static str, String> {
        self.fields
    }
}

impl Visit for ToScopeVisitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name(), value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name(), format!("{value:?}"));
    }
}
