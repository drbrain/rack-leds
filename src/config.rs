mod device;

use serde::{Deserialize, Serialize};

pub use device::Device;

#[derive(Deserialize, Serialize)]
pub struct Config {
    devices: Vec<Device>,
}

impl From<Config> for Vec<crate::device::Device> {
    fn from(config: Config) -> Self {
        config
            .devices
            .into_iter()
            .map(|device| device.into())
            .collect()
    }
}
