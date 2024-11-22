use std::sync::{Arc, Mutex};

use tracing_subscriber::{filter::Directive, EnvFilter};

use crate::ReloadHandle;

#[derive(Clone)]
pub struct Reloadable {
    handle: ReloadHandle,
    default: Directive,
    directives: Arc<Mutex<Vec<Directive>>>,
}

impl Reloadable {
    pub fn new(handle: ReloadHandle, default: Directive, mut directives: Vec<Directive>) -> Self {
        directives.sort_by_cached_key(|directive| directive.to_string());

        let directives = Arc::new(Mutex::new(directives));

        Self {
            handle,
            default,
            directives,
        }
    }

    pub fn delete(&self, index: usize) {
        let updated = {
            let mut directives = self.directives.lock().unwrap();

            directives.remove(index);

            directives.clone()
        };

        self.update_filter(updated);
    }

    pub fn directives(&self) -> Vec<Directive> {
        let directives = {
            let directives = self.directives.lock().unwrap();

            directives.clone()
        };

        directives
    }

    pub fn add(&self, directive: Directive) {
        let directives = {
            let mut directives = self.directives.lock().unwrap();

            directives.push(directive);

            directives.sort_by_cached_key(|directive| directive.to_string());

            directives.clone()
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
