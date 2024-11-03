mod switch;

use std::{
    fmt::Display,
    sync::{Mutex, OnceLock},
};

use eyre::Result;
pub use switch::Switch;

use crate::{collector::prometheus, Update};

static ID: OnceLock<Mutex<u64>> = OnceLock::new();

type Id = u64;

fn next_id() -> Id {
    let id_mutex = ID.get_or_init(|| Mutex::new(0));

    let mut guard = id_mutex.lock().unwrap();

    let id = *guard;

    *guard += 1;

    id
}

#[derive(Debug)]
pub enum Device {
    Switch { id: Id, device: Switch },
}

impl Device {
    pub fn switch(switch: Switch) -> Self {
        Device::Switch {
            id: next_id(),
            device: switch,
        }
    }

    pub async fn update(&self, connection: &prometheus::Connection) -> Result<Update> {
        let update = match self {
            Device::Switch { device: switch, .. } => {
                let device = switch.update(connection).await?;
                let layout = switch.layout(connection).await?;
                Update::Switch { layout, device }
            }
        };

        Ok(update)
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Device::Switch { id, device } => {
                f.write_str(&format!("switch {id} ({})", device.address()))
            }
        }
    }
}
