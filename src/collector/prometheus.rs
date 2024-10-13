use eyre::{Context, OptionExt, Result};
use itertools::Itertools;
use prometheus_http_query::{response::PromqlResult, Client};
use std::{fmt::Display, time::Duration};
use tokio::{sync::watch, task::JoinHandle, time};
use tracing::{debug, instrument, trace};

use crate::{collector::Update, device::Device};

pub struct Prometheus {
    client: Client,
    timeout: i64,
    period: Duration,
    devices: Vec<Device>,
    update: watch::Sender<Vec<Update>>,
}

impl Prometheus {
    pub fn new(
        prometheus_url: &str,
        period: Duration,
        timeout: Duration,
        devices: Vec<Device>,
    ) -> Result<Self> {
        debug!(url = %prometheus_url, ?period, ?timeout, "creating client");
        let client =
            Client::try_from(prometheus_url).wrap_err("unable to create prometheus client")?;

        let timeout = timeout
            .as_millis()
            .try_into()
            .wrap_err_with(|| format!("timeout {timeout:?} is too long"))?;

        let (update, _) = watch::channel(vec![]);

        Ok(Self {
            client,
            timeout,
            period,
            devices,
            update,
        })
    }

    pub fn collect(self) -> (watch::Receiver<Vec<Update>>, JoinHandle<Result<()>>) {
        let update = self.update.subscribe();

        let collector = tokio::spawn(self.collector());

        (update, collector)
    }

    #[instrument(skip_all, fields(target = %self.client.base_url().to_string()))]
    async fn collector(self) -> Result<()> {
        let mut interval = time::interval(self.period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        debug!(period = ?interval.period(), "started");

        loop {
            interval.tick().await;

            debug!("updating devices");

            let mut updates = Vec::with_capacity(self.devices.len());

            for device in self.devices.iter() {
                let update = device.update(&self).await?;
                updates.push(update);
            }

            self.update.send_replace(updates);
        }
    }

    #[instrument(skip_all, fields(%query, %label))]
    pub async fn get_label(&self, query: impl Display, label: impl Display) -> Result<String> {
        let result = self.query(query).await?;

        let value = result
            .data()
            .as_vector()
            .ok_or_eyre("Non-vector query result")?
            .iter()
            .next()
            .ok_or_eyre("Nothing matched")?
            .metric()
            .get(&label.to_string())
            .ok_or_eyre(format!("Could not find label {label}"))?;

        trace!(?value);

        Ok(value.to_string())
    }

    #[instrument(skip_all, fields(%query))]
    pub async fn get_values(&self, query: impl Display) -> Result<Vec<u64>> {
        let values: Vec<_> = self
            .query(query)
            .await?
            .data()
            .as_vector()
            .ok_or_eyre("Non-vector query result")?
            .iter()
            .sorted_by_key(|v| {
                v.metric()
                    .get("ifIndex")
                    .unwrap_or(&"0".to_string())
                    .parse()
                    .unwrap_or(0)
            })
            .map(|v| v.sample().value() as u64)
            .collect();

        trace!(?values);

        Ok(values)
    }

    #[instrument(skip_all, fields(%query, %label))]
    pub async fn get_values_with_label(
        &self,
        query: impl Display,
        label: impl Display,
    ) -> Result<Vec<(u64, Option<String>)>> {
        let label = label.to_string();

        let values: Vec<_> = self
            .query(query)
            .await?
            .data()
            .as_vector()
            .ok_or_eyre("Non-vector query result")?
            .iter()
            .map(|v| (v.sample().value() as u64, v.metric().get(&label).cloned()))
            .collect();

        trace!(?values);

        Ok(values)
    }

    async fn query(&self, query: impl Display) -> Result<PromqlResult> {
        Ok(self.client.query(query).timeout(self.timeout).get().await?)
    }
}
