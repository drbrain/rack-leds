use std::{collections::HashMap, sync::Arc};

use crate::device::{Device, Id};

pub struct Devices {
    columns: Vec<Vec<Id>>,
    devices: HashMap<Id, Arc<Device>>,
}
impl Devices {
    pub fn new(columns: Vec<Vec<Id>>, devices: HashMap<Id, Arc<Device>>) -> Self {
        Self { columns, devices }
    }

    pub fn columns(&self) -> &[Vec<Id>] {
        &self.columns
    }

    pub fn devices(&self) -> HashMap<Id, Arc<Device>> {
        self.devices.clone()
    }
}
