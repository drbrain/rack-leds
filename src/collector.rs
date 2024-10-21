mod absolute;
mod diff;
mod prometheus;

pub use absolute::Absolute;
pub use diff::Diff;
use eyre::{Context, Result};
pub use prometheus::Prometheus;
use tokio::{sync::watch, task::JoinHandle};

use crate::{Args, Update};

pub struct Collector {
    collector: JoinHandle<Result<()>>,
    update: watch::Receiver<Vec<Update>>,
}

impl Collector {
    pub fn new(args: &Args) -> Result<Self> {
        let devices = args.config()?.into();

        let prometheus = Prometheus::new(&args.source, args.period(), args.timeout(), devices)?;

        let (update, collector) = prometheus.collect();

        Ok(Self { collector, update })
    }

    pub fn subscribe(&self) -> watch::Receiver<Vec<Update>> {
        self.update.clone()
    }

    pub async fn wait(self) -> Result<()> {
        self.collector.await?.wrap_err("collector error")
    }
}
