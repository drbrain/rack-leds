use serde::{Deserialize, Serialize};

use crate::device::{self, Switch};

#[derive(Deserialize, Serialize)]
pub struct Config {
    devices: Vec<Device>,
}

impl From<Config> for Vec<device::Device> {
    fn from(config: Config) -> Self {
        config.devices.iter().map(|device| device.into()).collect()
    }
}

#[derive(Deserialize, Serialize)]
enum Device {
    Switch { address: String },
}

impl From<&Device> for crate::device::Device {
    fn from(device: &Device) -> Self {
        match device {
            Device::Switch { address } => {
                device::Device::Switch(Switch::new(&format!("instance=\"{address}\"")))
            }
        }
    }
}
