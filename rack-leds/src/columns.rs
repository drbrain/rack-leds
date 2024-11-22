use crate::Column;

#[derive(Clone)]
pub struct Columns {
    columns: Vec<Column>,
}

impl Columns {
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
    }

    pub fn columns(&self) -> Iter<'_> {
        Iter {
            columns: &self.columns,
            index: 0,
        }
    }
}

pub struct Iter<'a> {
    columns: &'a Vec<Column>,
    index: usize,
}

impl Iterator for Iter<'_> {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.columns.get(self.index);

        self.index += 1;

        current.cloned()
    }
}
