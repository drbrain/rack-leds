mod manager;

use deadpool::managed::Object;
use eyre::{Context, OptionExt, Result};
use itertools::Itertools;
pub use manager::Manager;
use prometheus_http_query::{response::PromqlResult, Client};
use std::fmt::Display;
use tracing::{debug, instrument, trace};

pub type Connection = Object<Manager>;

pub struct Prometheus {
    client: Client,
    timeout: i64,
}

impl Prometheus {
    pub fn new(url: &str, timeout: i64) -> Result<Self> {
        debug!(url = %url, ?timeout, "creating client");
        let client =
            Client::try_from(url).wrap_err(format!("Unable to create client for {url}"))?;

        Ok(Self { client, timeout })
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

impl std::fmt::Debug for Prometheus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Prometheus")
            .field("url", &self.client.base_url())
            .field("timeout", &self.timeout)
            .finish()
    }
}
