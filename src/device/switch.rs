use eyre::Result;
use rand::{distributions::Uniform, prelude::Distribution, rngs::SmallRng, seq::SliceRandom, Rng};
use tracing::instrument;

use crate::{
    collector::{prometheus, Absolute, Diff},
    device::Id,
    simulator::Simulated,
    update, Layout,
};

const PORTS: [usize; 4] = [5, 8, 10, 18];
const DISABLED_THRESHOLD: f64 = 0.1;

#[derive(Clone)]
pub struct Switch {
    address: String,
    labels: String,
    receive: Diff<Vec<u64>>,
    receive_query: String,
    transmit: Diff<Vec<u64>>,
    transmit_query: String,
    poe: Absolute<Vec<u64>>,
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

    // TODO: Return simulation data, let Device::simulate set id
    pub fn simulate(&self, id: Id, rng: &mut SmallRng, traffic: &Uniform<u64>) -> Simulated {
        let ports = PORTS.choose(rng).unwrap();
        let mut weights = Vec::with_capacity(*ports);

        for _ in 0..*ports {
            if rng.gen::<f64>() < DISABLED_THRESHOLD {
                weights.push(Uniform::new_inclusive(0, 0));
            } else {
                let low = traffic.sample(rng);
                let high = 1 + low + traffic.sample(rng);
                weights.push(Uniform::new(low, high));
            }
        }

        Simulated::Switch {
            id,
            ports: *ports,
            weights,
        }
    }

    #[instrument(level="debug", skip_all, ret, fields(labels = ?self.labels))]
    pub async fn update(&self, connection: &prometheus::Connection) -> Result<update::Switch> {
        self.receive.update(
            connection
                .get_values(&self.receive_query)
                .await?
                .iter()
                .map(|v| *v as u64)
                .collect(),
        );
        let receive_difference = self.receive.difference();

        self.transmit.update(
            connection
                .get_values(&self.transmit_query)
                .await?
                .iter()
                .map(|v| *v as u64)
                .collect(),
        );
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
