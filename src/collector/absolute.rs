use std::sync::{Arc, RwLock};

use crate::collector::Data;

#[derive(Clone, Default)]
pub struct Absolute<T> {
    inner: Arc<RwLock<T>>,
}

impl<T: Data + Clone> Absolute<T> {
    pub fn len(&self) -> usize {
        let inner = self.inner.read().unwrap();

        inner.len()
    }

    pub fn update(&self, update: T) {
        let mut inner = self.inner.write().unwrap();

        *inner = update;
    }

    pub fn value(&self) -> T {
        let inner = self.inner.read().unwrap();

        inner.clone()
    }
}

impl<T> From<T> for Absolute<T> {
    fn from(values: T) -> Self {
        Self {
            inner: Arc::new(values.into()),
        }
    }
}

impl From<&Absolute<Vec<u64>>> for Vec<u64> {
    fn from(absolute: &Absolute<Vec<u64>>) -> Self {
        let inner = absolute.inner.read().unwrap();

        inner.clone()
    }
}
