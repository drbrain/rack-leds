mod switch;

use std::fmt::Display;

use eyre::Result;
pub use switch::Switch;

use crate::{collector::prometheus, Update};

pub enum Device {
    Switch(Switch),
}

impl Device {
    pub async fn update(&self, connection: &prometheus::Connection) -> Result<Update> {
        let update = match self {
            Device::Switch(switch) => {
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
            Device::Switch(switch) => f.write_str(&format!("switch {}", switch.address())),
        }
    }
}
