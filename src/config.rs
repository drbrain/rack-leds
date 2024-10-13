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
    Switch {
        address: String,
        receive: Option<String>,
        transmit: Option<String>,
        poe: Option<String>,
    },
}

impl From<&Device> for crate::device::Device {
    fn from(device: &Device) -> Self {
        match device {
            Device::Switch {
                address,
                receive,
                transmit,
                poe,
            } => {
                let labels = format!("instance=\"{address}\"");

                let receive_query = if let Some(receive) = receive {
                    receive.clone()
                } else {
                    format!(
                        "sum(rate(ifHCInOctets{{{}, ifAlias=~\"(Port|SFP) .*\"}}[1m])) by (ifIndex)",
                        labels
                    )
                };

                let transmit_query = if let Some(transmit) = transmit {
                    transmit.clone()
                } else {
                    format!(
                        "sum(rate(ifHCOutOctets{{{}, ifAlias=~\"(Port|SFP) .*\"}}[1m])) by (ifIndex)",
                        labels
                    )
                };

                let poe_query = if let Some(poe) = poe {
                    poe.clone()
                } else {
                    format!("unpoller_device_port_poe_amperes{{{}}}", labels)
                };

                device::Device::Switch(Switch::new(
                    &labels,
                    &receive_query,
                    &transmit_query,
                    &poe_query,
                ))
            }
        }
    }
}
