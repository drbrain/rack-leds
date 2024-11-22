use serde::{Deserialize, Serialize};

use crate::config::Device;

#[derive(Deserialize, Serialize)]
pub struct Column {
    devices: Vec<Device>,
}

impl Column {
    pub fn devices(&self) -> &[Device] {
        &self.devices
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }
}
