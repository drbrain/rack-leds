mod absolute;
mod diff;
pub mod prometheus;

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub use absolute::Absolute;
use deadpool::managed::Pool;
pub use diff::Diff;
use eyre::{eyre, Context, Result};
pub use prometheus::Prometheus;
use tokio::{sync::watch, task::JoinSet, time};
use tracing::debug;

use crate::{device::Device, Args, Update};

pub type UpdateReceiver = watch::Receiver<(Vec<Update>, SystemTime)>;
pub type UpdateSender = watch::Sender<(Vec<Update>, SystemTime)>;

pub struct Collector {
    devices: Vec<Arc<Device>>,
    period: Duration,
    pool: Pool<prometheus::Manager>,
    update_sender: UpdateSender,
}

impl Collector {
    pub fn new(args: &Args) -> Result<Self> {
        let (update_sender, _) = watch::channel((vec![], UNIX_EPOCH));

        let devices: Vec<Device> = args.config()?.into();
        let devices = devices.into_iter().map(|device| device.into()).collect();

        //let prometheus = Prometheus::new(&args.source, args.period(), args.timeout(), devices)?;

        let pool = Pool::builder(prometheus::Manager::new(args)?)
            .build()
            .wrap_err(format!("Unable to create pool for {}", args.source))?;

        Ok(Self {
            devices,
            period: args.period(),
            pool,
            update_sender,
        })
    }

    pub async fn run(self) -> Result<()> {
        let mut interval = time::interval(self.period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        debug!(period = ?interval.period(), "started");

        loop {
            interval.tick().await;

            debug!("updating devices");

            let mut update_tasks = JoinSet::new();

            for device in self.devices.iter() {
                let device = device.clone();
                let pool = self.pool.clone();

                update_tasks
                    .build_task()
                    .name(&format!("update {}", device))
                    .spawn(async move {
                        match pool.get().await {
                            Ok(conn) => device.update(&conn).await,
                            Err(e) => Err(eyre!(e)).wrap_err(format!(
                                "retrieving connection for {}",
                                pool.manager().url()
                            )),
                        }
                    })?;
            }

            let mut updates = Vec::with_capacity(self.devices.len());

            while let Some(result) = update_tasks.join_next().await {
                updates.push(result??);
            }

            self.update_sender
                .send_replace((updates, SystemTime::now()));
        }
    }

    pub fn run_on(self, join_set: &mut JoinSet<Result<()>>) -> Result<()> {
        join_set
            .build_task()
            .name("collector outer")
            .spawn(async move { self.run().await })?;

        Ok(())
    }

    pub fn subscribe(&self) -> UpdateReceiver {
        self.update_sender.subscribe()
    }
}
