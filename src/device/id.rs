use std::{
    fmt::Display,
    sync::{Mutex, OnceLock},
};

static ID: OnceLock<Mutex<u64>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u64);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn next_id() -> Id {
    let id_mutex = ID.get_or_init(|| Mutex::new(0));

    let mut guard = id_mutex.lock().unwrap();

    let id = *guard;

    *guard += 1;

    Id(id)
}
