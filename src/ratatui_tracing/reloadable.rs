use std::sync::{Arc, Mutex};

use tracing_subscriber::{filter::Directive, EnvFilter};

use crate::ratatui_tracing::ReloadHandle;

#[derive(Clone)]
pub struct Reloadable {
    handle: ReloadHandle,
    default: Directive,
    directives: Arc<Mutex<Vec<Directive>>>,
}

impl Reloadable {
    pub fn new(handle: ReloadHandle, default: Directive, directives: Vec<Directive>) -> Self {
        let directives = Arc::new(Mutex::new(directives));

        Self {
            handle,
            default,
            directives,
        }
    }

    pub fn delete(&self, index: usize) {
        let updated = {
            let mut guard = self.directives.lock().unwrap();

            guard.remove(index);

            guard.clone()
        };

        self.update_filter(updated);
    }

    pub fn directives(&self) -> Vec<Directive> {
        let directives = {
            let guard = self.directives.lock().unwrap();

            guard.clone()
        };

        directives
    }

    pub fn add(&self, directive: Directive) {
        let directives = {
            let mut guard = self.directives.lock().unwrap();

            guard.push(directive);

            guard.clone()
        };

        self.update_filter(directives);
    }

    fn update_filter(&self, updated: Vec<Directive>) {
        let filter = EnvFilter::builder()
            .with_default_directive(self.default.clone())
            .parse_lossy("");

        let filter = updated.iter().fold(filter, |filter, directive| {
            filter.add_directive(directive.clone())
        });

        self.handle.reload(filter).ok();
    }
}
