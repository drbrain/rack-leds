mod id;
mod switch;

use std::fmt::Display;

use eyre::Result;
use id::next_id;
pub use id::Id;
pub use switch::Switch;

use crate::{collector::prometheus, Update};

#[derive(Clone, Debug)]
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

    pub fn id(&self) -> Id {
        match self {
            Device::Switch { id, .. } => *id,
        }
    }

    pub async fn update(&self, connection: &prometheus::Connection) -> Result<Update> {
        let update = match self {
            Device::Switch { id, device: switch } => {
                let device = switch.update(connection).await?;
                let layout = switch.layout(connection).await?;

                Update::Switch {
                    id: *id,
                    layout,
                    device,
                }
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
