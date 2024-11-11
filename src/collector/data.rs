pub trait Data {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

impl Data for f64 {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        1
    }
}

impl Data for u64 {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        1
    }
}

impl Data for Vec<u64> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }
}
