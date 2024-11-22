use std::{collections::HashMap, sync::Arc};

use crate::{
    device::{Device, Id},
    Columns,
};

pub struct Devices {
    columns: Columns,
    devices: HashMap<Id, Arc<Device>>,
}
impl Devices {
    pub fn new(columns: Columns, devices: HashMap<Id, Arc<Device>>) -> Self {
        Self { columns, devices }
    }

    pub fn columns(&self) -> &Columns {
        &self.columns
    }

    pub fn devices(&self) -> HashMap<Id, Arc<Device>> {
        self.devices.clone()
    }
}
