use std::sync::RwLock;

#[derive(Default)]
pub struct Absolute {
    inner: RwLock<Vec<u64>>,
}

impl Absolute {
    pub fn len(&self) -> usize {
        let inner = self.inner.read().unwrap();

        inner.len()
    }

    pub fn update(&self, update: Vec<u64>) {
        let mut inner = self.inner.write().unwrap();

        *inner = update;
    }
}

impl From<Vec<u64>> for Absolute {
    fn from(values: Vec<u64>) -> Self {
        Self {
            inner: values.into(),
        }
    }
}

impl From<&Absolute> for Vec<u64> {
    fn from(absolute: &Absolute) -> Self {
        let inner = absolute.inner.read().unwrap();

        inner.clone()
    }
}
