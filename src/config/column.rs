use serde::{Deserialize, Serialize};

use crate::config::Device;

#[derive(Deserialize, Serialize)]
pub struct Column {
    devices: Vec<Device>,
}
