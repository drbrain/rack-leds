mod switch;

use eyre::Result;
pub use switch::Switch;

use crate::{collector::Prometheus, Update};

pub enum Device {
    Switch(Switch),
}

impl Device {
    pub async fn update(&self, prometheus: &Prometheus) -> Result<Update> {
        let update = match self {
            Device::Switch(switch) => Update::Switch(switch.update(prometheus).await?),
        };

        Ok(update)
    }
}
