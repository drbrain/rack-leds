use std::{cmp::Ordering, sync::RwLock};

#[derive(Default)]
pub struct Diff {
    inner: RwLock<Inner>,
}

impl Diff {
    pub fn difference(&self) -> Vec<u64> {
        let inner = self.inner.read().unwrap();

        inner.difference()
    }

    /// Updates this [`Diff`] only if the update does not match the current value
    pub fn update(&self, update: Vec<u64>) {
        let mut inner = self.inner.write().unwrap();

        inner.update(update)
    }
}

#[derive(Default)]
struct Inner {
    previous: Vec<u64>,
    current: Vec<u64>,
}

impl Inner {
    fn difference(&self) -> Vec<u64> {
        if self.previous.is_empty() {
            return vec![0; self.current.len()];
        };

        let mut previous = self.previous.clone();

        match previous.len().cmp(&self.current.len()) {
            Ordering::Less => previous.resize(self.current.len(), 0),
            Ordering::Equal => (),
            Ordering::Greater => previous.truncate(self.current.len()),
        }

        previous
            .iter()
            .zip(self.current.iter())
            .map(|(p, c)| c.saturating_sub(*p))
            .collect()
    }

    fn update(&mut self, update: Vec<u64>) {
        if self.current != update {
            self.previous = self.current.clone();
            self.current = update;
        }
    }
}
