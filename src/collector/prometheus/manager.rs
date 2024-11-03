use deadpool::managed;
use eyre::{Context, Error, Result};

use crate::Args;

use crate::collector::Prometheus;

pub struct Manager {
    timeout: i64,
    url: String,
}

impl Manager {
    pub fn new(args: &Args) -> Result<Self> {
        let timeout = args
            .timeout()
            .as_millis()
            .try_into()
            .wrap_err_with(|| format!("timeout {:?} is too long", args.timeout()))?;

        Ok(Self {
            timeout,
            url: args.source.clone(),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl managed::Manager for Manager {
    type Type = Prometheus;

    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Prometheus::new(&self.url, self.timeout)
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
