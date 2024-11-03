use eyre::Result;
use tracing::instrument;

use crate::{
    collector::{prometheus, Absolute, Diff},
    update, Layout,
};

#[derive(Clone)]
pub struct Switch {
    address: String,
    labels: String,
    receive: Diff,
    receive_query: String,
    transmit: Diff,
    transmit_query: String,
    poe: Absolute,
    poe_query: String,
}

impl Switch {
    pub fn new(
        address: &str,
        labels: &str,
        receive_query: &str,
        transmit_query: &str,
        poe_query: &str,
    ) -> Self {
        Self {
            address: address.to_string(),
            labels: labels.to_string(),
            receive: Default::default(),
            receive_query: receive_query.into(),
            transmit: Default::default(),
            transmit_query: transmit_query.into(),
            poe: Default::default(),
            poe_query: poe_query.into(),
        }
    }

    pub fn address(&self) -> String {
        self.address.clone()
    }

    #[instrument(skip_all, fields(labels = ?self.labels))]
    pub async fn layout(&self, connection: &prometheus::Connection) -> Result<Layout> {
        Layout::new(connection, &self.labels).await
    }

    #[instrument(level="debug", skip_all, ret, fields(labels = ?self.labels))]
    pub async fn update(&self, connection: &prometheus::Connection) -> Result<update::Switch> {
        self.receive
            .update(connection.get_values(&self.receive_query).await?);
        let receive_difference = self.receive.difference();

        self.transmit
            .update(connection.get_values(&self.transmit_query).await?);
        let transmit_difference = self.transmit.difference();

        self.update_poe(connection).await?;

        Ok(update::Switch::new(
            receive_difference,
            transmit_difference,
            (&self.poe).into(),
        ))
    }

    async fn update_poe(&self, connection: &prometheus::Connection) -> Result<()> {
        let mut poe = vec![0; self.receive.len()];

        connection
            .get_values_with_label(&self.poe_query, "port_num")
            .await?
            .iter()
            .map(|(v, l)| {
                (
                    v,
                    l.clone().and_then(|l| l.parse::<usize>().ok()).unwrap_or(0),
                )
            })
            .for_each(|(v, p)| poe[p] = *v);

        self.poe.update(poe);

        Ok(())
    }
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("address", &self.address)
            .field("labels", &self.labels)
            .finish()
    }
}
