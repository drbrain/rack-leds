mod column;
mod device;

use std::collections::HashMap;

use column::Column;
use serde::{Deserialize, Serialize};

pub use device::Device;

use crate::Devices;

#[derive(Deserialize, Serialize)]
pub struct Config {
    columns: Vec<Column>,
}

impl From<Config> for Devices {
    fn from(config: Config) -> Self {
        let (columns, devices) = config.columns.iter().fold(
            (Vec::with_capacity(config.columns.len()), HashMap::default()),
            |(mut columns, mut devices), column| {
                let (column, _) = column.devices().iter().fold(
                    (Vec::with_capacity(column.len()), &mut devices),
                    |(mut column, devices), device| {
                        let device: crate::device::Device = device.into();

                        column.push(device.id());
                        devices.insert(device.id(), device.into());

                        (column, devices)
                    },
                );

                columns.push(column);

                (columns, devices)
            },
        );

        Self::new(columns, devices)
    }
}
