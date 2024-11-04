use crate::device::Id;

#[derive(Clone)]
pub struct Column {
    ids: Vec<Id>,
}

impl Column {
    pub fn new(ids: Vec<Id>) -> Self {
        Self { ids }
    }

    pub fn ids(&self) -> Iter<'_> {
        Iter {
            ids: &self.ids,
            index: 0,
        }
    }
}

pub struct Iter<'a> {
    ids: &'a Vec<Id>,
    index: usize,
}

impl Iterator for Iter<'_> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.ids.get(self.index);

        self.index += 1;

        current.cloned()
    }
}
