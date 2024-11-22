use eyre::Result;
use rand::{distributions::Uniform, prelude::SmallRng};
use tracing::instrument;

use crate::{
    collector::{prometheus, Absolute, Diff},
    device::Id,
    simulator::Simulated,
    update, Layout,
};

#[derive(Clone)]
pub struct AccessPoint {
    address: String,
    name: String,
    channel_utilization_24_ghz: Absolute<f64>,
    channel_utilization_24_ghz_query: String,
    channel_utilization_5_ghz: Absolute<f64>,
    channel_utilization_5_ghz_query: String,
    receive_ap: Diff<u64>,
    receive_ap_query: String,
    receive_wan_24_ghz: Diff<u64>,
    receive_wan_24_ghz_query: String,
    receive_wan_5_ghz: Diff<u64>,
    receive_wan_5_ghz_query: String,
    stations_24_ghz: Absolute<u64>,
    stations_24_ghz_query: String,
    stations_5_ghz: Absolute<u64>,
    stations_5_ghz_query: String,
    transmit_ap: Diff<u64>,
    transmit_ap_query: String,
    transmit_wan_24_ghz: Diff<u64>,
    transmit_wan_24_ghz_query: String,
    transmit_wan_5_ghz: Diff<u64>,
    transmit_wan_5_ghz_query: String,
}

impl AccessPoint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: String,
        name: String,
        channel_utilization_24_ghz_query: String,
        channel_utilization_5_ghz_query: String,
        receive_ap_query: String,
        receive_wan_24_ghz_query: String,
        receive_wan_5_ghz_query: String,
        stations_24_ghz_query: String,
        stations_5_ghz_query: String,
        transmit_ap_query: String,
        transmit_wan_24_ghz_query: String,
        transmit_wan_5_ghz_query: String,
    ) -> Self {
        Self {
            address,
            name,
            channel_utilization_24_ghz: Default::default(),
            channel_utilization_24_ghz_query,
            channel_utilization_5_ghz: Default::default(),
            channel_utilization_5_ghz_query,
            receive_ap: Default::default(),
            receive_ap_query,
            receive_wan_24_ghz: Default::default(),
            receive_wan_24_ghz_query,
            receive_wan_5_ghz: Default::default(),
            receive_wan_5_ghz_query,
            stations_24_ghz: Default::default(),
            stations_24_ghz_query,
            stations_5_ghz: Default::default(),
            stations_5_ghz_query,
            transmit_ap: Default::default(),
            transmit_ap_query,
            transmit_wan_24_ghz: Default::default(),
            transmit_wan_24_ghz_query,
            transmit_wan_5_ghz: Default::default(),
            transmit_wan_5_ghz_query,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn layout(&self, _connection: &prometheus::Connection) -> Result<Layout> {
        Ok(Layout::AccessPoint)
    }

    pub fn simulate(&self, _id: Id, _rng: &mut SmallRng, _traffic: &Uniform<u64>) -> Simulated {
        todo!()
    }

    #[instrument(level="debug", skip_all, ret, fields(address = ?self.address))]
    pub async fn update(&self, connection: &prometheus::Connection) -> Result<update::AccessPoint> {
        self.channel_utilization_24_ghz.update(
            *connection
                .get_values(&self.channel_utilization_24_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0),
        );

        self.channel_utilization_5_ghz.update(
            *connection
                .get_values(&self.channel_utilization_5_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0),
        );

        self.receive_ap.update(
            *connection
                .get_values(&self.receive_ap_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );
        let receive_ap_difference = self.receive_ap.difference();

        self.receive_wan_24_ghz.update(
            *connection
                .get_values(&self.receive_wan_24_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );
        let receive_wan_24_ghz_difference = self.receive_wan_24_ghz.difference();

        self.receive_wan_5_ghz.update(
            *connection
                .get_values(&self.receive_wan_5_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );
        let receive_wan_5_ghz_difference = self.receive_wan_5_ghz.difference();

        self.stations_24_ghz.update(
            *connection
                .get_values(&self.stations_24_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );

        self.stations_5_ghz.update(
            *connection
                .get_values(&self.stations_5_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );

        self.transmit_ap.update(
            *connection
                .get_values(&self.transmit_ap_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );
        let transmit_ap_difference = self.transmit_ap.difference();

        self.transmit_wan_24_ghz.update(
            *connection
                .get_values(&self.transmit_wan_24_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );
        let transmit_wan_24_ghz_difference = self.transmit_wan_24_ghz.difference();

        self.transmit_wan_5_ghz.update(
            *connection
                .get_values(&self.transmit_wan_5_ghz_query)
                .await?
                .first()
                .unwrap_or(&0.0) as u64,
        );
        let transmit_wan_5_ghz_difference = self.transmit_wan_5_ghz.difference();

        Ok(update::AccessPoint::new(
            (self.channel_utilization_24_ghz.value() * 100.0) as u64,
            (self.channel_utilization_5_ghz.value() * 100.0) as u64,
            receive_ap_difference,
            receive_wan_24_ghz_difference,
            receive_wan_5_ghz_difference,
            self.stations_24_ghz.value(),
            self.stations_5_ghz.value(),
            transmit_ap_difference,
            transmit_wan_24_ghz_difference,
            transmit_wan_5_ghz_difference,
        ))
    }
}

impl std::fmt::Debug for AccessPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("address", &self.address)
            .field("name", &self.name)
            .finish()
    }
}
