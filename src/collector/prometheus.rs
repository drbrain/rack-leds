use eyre::{Context, OptionExt, Result};
use prometheus_http_query::Client;
use std::{fmt::Display, time::Duration};
use tokio::{sync::watch, task::JoinHandle, time};
use tracing::{debug, instrument, trace};

use crate::collector::{Diff, Update};

const RECEIVE_BYTES: &str =
    r#"sum(unpoller_device_port_receive_bytes_total{name="Office nook switch"}) by (port_name)"#;
const TRANSMIT_BYTES: &str =
    r#"sum(unpoller_device_port_transmit_bytes_total{name="Office nook switch"}) by (port_name)"#;

pub struct Prometheus {
    client: Client,
    timeout: i64,
    period: Duration,
    receive_bytes: Diff,
    transmit_bytes: Diff,
    update: watch::Sender<Update>,
}

impl Prometheus {
    pub fn new(prometheus_url: &str, period: Duration, timeout: Duration) -> Result<Self> {
        debug!(url = %prometheus_url, ?period, ?timeout, "creating client");
        let client =
            Client::try_from(prometheus_url).wrap_err("unable to create prometheus client")?;

        let timeout = timeout
            .as_millis()
            .try_into()
            .wrap_err_with(|| format!("timeout {timeout:?} is too long"))?;

        let (update, _) = watch::channel(Update::empty());

        Ok(Self {
            client,
            timeout,
            period,
            receive_bytes: Diff::empty(),
            transmit_bytes: Diff::empty(),
            update,
        })
    }

    pub fn collect(self) -> (watch::Receiver<Update>, JoinHandle<Result<()>>) {
        let update = self.update.subscribe();

        let collector = tokio::spawn(self.collector());

        (update, collector)
    }

    #[instrument(skip_all, fields(target = %self.client.base_url().to_string()))]
    async fn collector(mut self) -> Result<()> {
        let mut interval = time::interval(self.period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        debug!(period = ?interval.period(), "started");

        loop {
            interval.tick().await;

            self.receive_bytes.update(self.query(RECEIVE_BYTES).await?);

            self.transmit_bytes
                .update(self.query(TRANSMIT_BYTES).await?);

            let receive_difference = self.receive_bytes.difference();
            let transmit_difference = self.transmit_bytes.difference();

            trace!(
                receive = ?receive_difference,
                transmit = ?transmit_difference,
                "updated"
            );

            let update = Update::new(receive_difference, transmit_difference);

            self.update.send_replace(update);
        }
    }

    #[instrument(skip_all, fields(%query))]
    async fn query(&self, query: impl Display) -> Result<Vec<u64>> {
        let values: Vec<_> = self
            .client
            .query(query)
            .timeout(self.timeout)
            .get()
            .await?
            .data()
            .as_vector()
            .ok_or_eyre("Non-vector query result")?
            .iter()
            .map(|v| v.sample().value() as u64)
            .collect();

        trace!(?values);

        Ok(values)
    }
}
