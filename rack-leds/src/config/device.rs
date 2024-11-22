use serde::{Deserialize, Serialize};

use crate::device::{AccessPoint, Switch};

#[derive(Deserialize, Serialize)]
pub enum Device {
    AccessPoint {
        address: String,
        name: String,
        channel_utilization_24_ghz: Option<String>,
        channel_utilization_5_ghz: Option<String>,
        receive_ap: Option<String>,
        receive_wan_24_ghz: Option<String>,
        receive_wan_5_ghz: Option<String>,
        stations_24_ghz: Option<String>,
        stations_5_ghz: Option<String>,
        transmit_ap: Option<String>,
        transmit_wan_24_ghz: Option<String>,
        transmit_wan_5_ghz: Option<String>,
    },
    Switch {
        address: String,
        receive: Option<String>,
        transmit: Option<String>,
        poe: Option<String>,
    },
}

impl From<Device> for crate::device::Device {
    fn from(device: Device) -> Self {
        (&device).into()
    }
}

impl From<&Device> for crate::device::Device {
    fn from(device: &Device) -> Self {
        match device {
            Device::AccessPoint {
                address,
                name,
                channel_utilization_24_ghz,
                channel_utilization_5_ghz,
                receive_ap,
                receive_wan_24_ghz,
                receive_wan_5_ghz,
                stations_5_ghz,
                stations_24_ghz,
                transmit_ap,
                transmit_wan_24_ghz,
                transmit_wan_5_ghz,
            } => {
                let address_label = format!("instance=\"{address}\"");
                let name_label = format!("name=\"{name}\"");

                let channel_utilization_24_ghz_query =
                    channel_utilization_24_ghz.clone().unwrap_or_else(|| {
                        format!(
                            "unpoller_device_radio_channel_utilization_total_ratio{{{name_label}, radio=\"ng\"}}",
                        )
                    });

                let channel_utilization_5_ghz_query =
                    channel_utilization_5_ghz.clone().unwrap_or_else(|| {
                        format!(
                            "unpoller_device_radio_channel_utilization_total_ratio{{{name_label}, radio=\"na\"}}",
                        )
                    });

                let receive_ap_query = receive_ap.clone().unwrap_or_else(|| {
                    format!("sum(rate(ifHCInOctets{{{address_label}, ifName=\"eth0\"}}[1m]))",)
                });

                let receive_wan_24_ghz_query = receive_wan_24_ghz.clone().unwrap_or_else(|| {
                    format!("sum(rate(ifHCInOctets{{{address_label}, ifName=\"wifi1\"}}[1m]))",)
                });

                let receive_wan_5_ghz_query = receive_wan_5_ghz.clone().unwrap_or_else(|| {
                    format!("sum(rate(ifHCInOctets{{{address_label}, ifName=\"wifi0\"}}[1m]))",)
                });

                let stations_24_ghz_query = stations_24_ghz.clone().unwrap_or_else(|| {
                    format!("sum(unpoller_device_radio_stations{{{name_label}, radio=\"ng\"}})",)
                });

                let stations_5_ghz_query = stations_5_ghz.clone().unwrap_or_else(|| {
                    format!("sum(unpoller_device_radio_stations{{{name_label}, radio=\"na\"}})",)
                });

                let transmit_ap_query = transmit_ap.clone().unwrap_or_else(|| {
                    format!("sum(rate(ifHCOutOctets{{{address_label}, ifName=\"eth0\"}}[1m]))",)
                });

                let transmit_wan_24_ghz_query = transmit_wan_24_ghz.clone().unwrap_or_else(|| {
                    format!("sum(rate(ifHCOutOctets{{{address_label}, ifName=\"wifi1\"}}[1m]))",)
                });

                let transmit_wan_5_ghz_query = transmit_wan_5_ghz.clone().unwrap_or_else(|| {
                    format!("sum(rate(ifHCOutOctets{{{address_label}, ifName=\"wifi0\"}}[1m]))",)
                });

                crate::device::Device::access_point(AccessPoint::new(
                    address.clone(),
                    name.clone(),
                    channel_utilization_24_ghz_query,
                    channel_utilization_5_ghz_query,
                    receive_ap_query,
                    receive_wan_24_ghz_query,
                    receive_wan_5_ghz_query,
                    stations_24_ghz_query,
                    stations_5_ghz_query,
                    transmit_ap_query,
                    transmit_wan_24_ghz_query,
                    transmit_wan_5_ghz_query,
                ))
            }
            Device::Switch {
                address,
                receive,
                transmit,
                poe,
            } => {
                let labels = format!("instance=\"{address}\"");

                let receive_query =
                    receive.clone()
                .unwrap_or_else(|| {
                    format!(
                        "sum(rate(ifHCInOctets{{{}, ifAlias=~\"(Port|SFP) .*\"}}[1m])) by (ifIndex)",
                        labels
                    )
                });

                let transmit_query =
                    transmit.clone().unwrap_or_else(||
                    {
                    format!(
                        "sum(rate(ifHCOutOctets{{{}, ifAlias=~\"(Port|SFP) .*\"}}[1m])) by (ifIndex)",
                        labels
                    )
                });

                let poe_query = poe
                    .clone()
                    .unwrap_or_else(|| format!("unpoller_device_port_poe_amperes{{{}}}", labels));

                crate::device::Device::switch(Switch::new(
                    address,
                    &labels,
                    &receive_query,
                    &transmit_query,
                    &poe_query,
                ))
            }
        }
    }
}
