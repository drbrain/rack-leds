pub struct Update {
    receive: Vec<u64>,
    transmit: Vec<u64>,
}

impl Update {
    pub fn empty() -> Self {
        Self {
            receive: vec![],
            transmit: vec![],
        }
    }

    pub fn new(receive: Vec<u64>, transmit: Vec<u64>) -> Self {
        Self { receive, transmit }
    }
}
