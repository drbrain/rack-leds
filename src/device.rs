mod id;
mod switch;

use std::fmt::Display;

use eyre::Result;
use id::next_id;
pub use id::Id;
pub use switch::Switch;

use crate::{collector::prometheus, Update};

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
