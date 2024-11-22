mod access_point;
mod id;
mod switch;

use std::fmt::Display;

pub use access_point::AccessPoint;
use eyre::Result;
use id::next_id;
pub use id::Id;
pub use switch::Switch;

use crate::{collector::prometheus, Update};

#[derive(Clone, Debug)]
pub enum Device {
    AccessPoint { id: Id, device: AccessPoint },
    Switch { id: Id, device: Switch },
}

impl Device {
    pub fn access_point(access_point: AccessPoint) -> Self {
        Device::AccessPoint {
            id: next_id(),
            device: access_point,
        }
    }

    pub fn switch(switch: Switch) -> Self {
        Device::Switch {
            id: next_id(),
            device: switch,
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Device::AccessPoint { id, .. } => *id,
            Device::Switch { id, .. } => *id,
        }
    }

    pub async fn update(&self, connection: &prometheus::Connection) -> Result<Update> {
        let update = match self {
            Device::AccessPoint {
                id,
                device: access_point,
            } => {
                let device = access_point.update(connection).await?;
                let layout = access_point.layout(connection).await?;

                Update::AccessPoint {
                    id: *id,
                    device,
                    layout,
                }
            }
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
            Device::AccessPoint { id, device } => {
                f.write_str(&format!("AP {id} ({})", device.name()))
            }
            Device::Switch { id, device } => {
                f.write_str(&format!("switch {id} ({})", device.address()))
            }
        }
    }
}
