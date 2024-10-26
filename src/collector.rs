mod absolute;
mod diff;
mod prometheus;

use std::time::SystemTime;

pub use absolute::Absolute;
pub use diff::Diff;
use eyre::{Context, Result};
pub use prometheus::Prometheus;
use tokio::{sync::watch, task::JoinHandle};

use crate::{Args, Update};

pub type UpdateReceiver = watch::Receiver<(Vec<Update>, SystemTime)>;

pub struct Collector {
    collector: JoinHandle<Result<()>>,
    update: UpdateReceiver,
}

impl Collector {
    pub fn new(args: &Args) -> Result<Self> {
        let devices = args.config()?.into();

        let prometheus = Prometheus::new(&args.source, args.period(), args.timeout(), devices)?;

        let (update, collector) = prometheus.collect();

        Ok(Self { collector, update })
    }

    pub fn subscribe(&self) -> UpdateReceiver {
        self.update.clone()
    }

    pub async fn wait(self) -> Result<()> {
        self.collector.await?.wrap_err("collector error")
    }
}
