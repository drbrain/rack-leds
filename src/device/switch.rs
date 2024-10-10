use eyre::Result;
use tracing::{instrument, trace};

use crate::{
    collector::{Diff, Prometheus},
    update,
};

pub struct Switch {
    labels: String,
    receive: Diff,
    transmit: Diff,
}

impl Switch {
    pub fn new(labels: &str) -> Self {
        Self {
            labels: labels.to_string(),
            receive: Default::default(),
            transmit: Default::default(),
        }
    }

    #[instrument(skip_all, fields(labels = ?self.labels))]
    pub async fn update(&self, client: &Prometheus) -> Result<update::Switch> {
        let receive_query = format!(
            "sum(rate(ifHCInOctets{{{}}}[1m])) by (ifIndex)",
            self.labels
        );
        self.receive.update(client.query(receive_query).await?);
        let receive_difference = self.receive.difference();

        let transmit_query = format!(
            "sum(rate(ifHCOutOctets{{{}}}[1m])) by (ifIndex)",
            self.labels
        );
        self.transmit.update(client.query(transmit_query).await?);
        let transmit_difference = self.transmit.difference();

        trace!(
            receive = ?receive_difference,
            transmit = ?transmit_difference,
            "updated"
        );

        Ok(update::Switch::new(receive_difference, transmit_difference))
    }
}
