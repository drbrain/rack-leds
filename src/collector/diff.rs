use std::cmp::Ordering;

pub struct Diff {
    previous: Vec<u64>,
    current: Vec<u64>,
}

impl Diff {
    pub fn empty() -> Self {
        Self {
            previous: vec![],
            current: vec![],
        }
    }

    pub fn difference(&self) -> Vec<u64> {
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

    /// Updates this [`Diff`] only if the update does not match the current value
    pub fn update(&mut self, update: Vec<u64>) {
        if self.current != update {
            self.previous = self.current.clone();
            self.current = update;
        }
    }
}
