use std::{
    cmp::Ordering,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::collector::Data;

#[derive(Clone, Default)]
pub struct Diff<T> {
    inner: Arc<RwLock<Inner<T>>>,
}

impl Diff<u64> {
    pub fn difference(&self) -> u64 {
        let inner = self.inner.read().unwrap();

        inner.difference()
    }

    pub fn update(&self, update: u64) {
        let mut inner = self.inner.write().unwrap();

        inner.update(update)
    }
}

impl Diff<Vec<u64>> {
    pub fn difference(&self) -> Vec<u64> {
        let inner = self.inner.read().unwrap();

        inner.difference()
    }

    pub fn len(&self) -> usize {
        let inner = self.inner.read().unwrap();

        inner.len()
    }

    pub fn update(&self, update: Vec<u64>) {
        let mut inner = self.inner.write().unwrap();

        inner.update(update)
    }
}

#[derive(Default)]
struct Inner<T> {
    previous: T,
    current: T,
}

impl Data for Inner<u64> {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        1
    }
}

impl Data for Inner<Vec<u64>> {
    fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    fn len(&self) -> usize {
        self.current.len()
    }
}

pub trait Diffable {
    type Item;

    fn difference(&self) -> Self::Item;
}

impl Diffable for RwLockReadGuard<'_, Inner<u64>> {
    type Item = u64;

    fn difference(&self) -> Self::Item {
        self.current.saturating_sub(self.previous)
    }
}

impl Diffable for RwLockReadGuard<'_, Inner<Vec<u64>>> {
    type Item = Vec<u64>;

    fn difference(&self) -> Self::Item {
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
}

pub trait Updatable {
    type Item;

    fn update(&mut self, update: Self::Item);
}

impl Updatable for RwLockWriteGuard<'_, Inner<u64>> {
    type Item = u64;

    fn update(&mut self, update: Self::Item) {
        self.previous = self.current;

        self.current = update;
    }
}

impl Updatable for RwLockWriteGuard<'_, Inner<Vec<u64>>> {
    type Item = Vec<u64>;

    fn update(&mut self, update: Self::Item) {
        if self.current != update {
            self.previous = self.current.clone();
            self.current = update;
        }
    }
}
