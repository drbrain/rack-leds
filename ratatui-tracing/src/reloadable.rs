use std::sync::{Arc, RwLock};

use tracing_subscriber::{filter::Directive, EnvFilter};

use crate::ReloadHandle;

/// [`ReloadHandle`] wrapper used by [`crate::widgets::EventLogState`] to edit the tracing layer
/// filter
///
/// A `Reloadable` wraps a lock for convenient cloning
#[derive(Clone)]
pub struct Reloadable {
    inner: Arc<RwLock<Inner>>,
}

impl Reloadable {
    pub(crate) fn new(
        handle: ReloadHandle,
        default: Directive,
        directives: Vec<Directive>,
    ) -> Self {
        let inner = Inner::new(handle, default, directives);

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn add(&mut self, directive: Directive) {
        let mut inner = self.inner.write().unwrap();

        inner.add(directive);
    }

    pub(crate) fn delete(&self, index: usize) {
        let mut inner = self.inner.write().unwrap();

        inner.delete(index);
    }

    pub(crate) fn directives(&self) -> Vec<Directive> {
        let inner = self.inner.read().unwrap();

        inner.directives()
    }
}

#[derive(Clone)]
struct Inner {
    handle: ReloadHandle,
    default: Directive,
    directives: Vec<Directive>,
}

impl Inner {
    fn new(handle: ReloadHandle, default: Directive, mut directives: Vec<Directive>) -> Self {
        directives.sort_by_cached_key(|directive| directive.to_string());

        Self {
            handle,
            default,
            directives,
        }
    }

    pub fn add(&mut self, directive: Directive) {
        self.directives.push(directive);

        self.directives
            .sort_by_cached_key(|directive| directive.to_string());

        self.update_filter();
    }

    fn delete(&mut self, index: usize) {
        self.directives.remove(index);

        self.update_filter();
    }

    fn directives(&self) -> Vec<Directive> {
        self.directives.clone()
    }

    fn update_filter(&self) {
        let filter = EnvFilter::builder()
            .with_default_directive(self.default.clone())
            .parse_lossy("");

        let filter = self.directives.iter().fold(filter, |filter, directive| {
            filter.add_directive(directive.clone())
        });

        self.handle.reload(filter).ok();
    }
}
