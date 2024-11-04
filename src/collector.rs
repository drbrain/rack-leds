mod absolute;
mod diff;
pub mod prometheus;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub use absolute::Absolute;
use deadpool::managed::Pool;
pub use diff::Diff;
use eyre::{eyre, Context, Result};
pub use prometheus::Prometheus;
use tokio::{sync::watch, task::JoinSet, time};
use tracing::{debug, error, info, instrument, trace};

use crate::{
    device::{Device, Id},
    Args, Devices, Update,
};

pub type UpdateReceiver = watch::Receiver<(HashMap<Id, Update>, SystemTime)>;
pub type UpdateSender = watch::Sender<(HashMap<Id, Update>, SystemTime)>;

pub struct Collector {
    devices: Vec<Arc<Device>>,
    period: Duration,
    pool: Pool<prometheus::Manager>,
    update_sender: UpdateSender,
}

impl Collector {
    pub fn new(args: &Args, devices: &Devices) -> Result<Self> {
        let (update_sender, _) = watch::channel((HashMap::default(), UNIX_EPOCH));

        let mut devices: Vec<Arc<Device>> = devices.devices().values().cloned().collect();

        devices.sort_by_cached_key(|device| device.id());

        debug!("devices: {:#?}", devices);

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

    #[instrument(name = "collector", skip_all)]
    pub async fn run(self) -> Result<()> {
        let mut interval = time::interval(self.period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        info!(period = ?interval.period(), "started");

        loop {
            interval.tick().await;

            debug!(count = self.devices.len(), "updating devices");

            let mut update_tasks = JoinSet::new();

            for device in self.devices.iter() {
                let device = device.clone();
                let pool = self.pool.clone();

                update_tasks
                    .build_task()
                    .name(&format!("update {}", device))
                    .spawn(async move { update(pool, device).await })?;
            }

            let mut updates = HashMap::with_capacity(self.devices.len());

            while let Some(result) = update_tasks.join_next().await {
                match result? {
                    Ok(update) => {
                        updates.insert(update.id(), update);
                    }
                    Err(e) => error!(?e, "device update error"),
                }
            }

            self.update_sender
                .send_replace((updates, SystemTime::now()));
        }
    }

    pub fn run_on(self, join_set: &mut JoinSet<Result<()>>) -> Result<()> {
        join_set
            .build_task()
            .name("collector")
            .spawn(async move { self.run().await })?;

        Ok(())
    }

    pub fn subscribe(&self) -> UpdateReceiver {
        self.update_sender.subscribe()
    }
}

#[instrument(skip_all, err, fields(url = pool.manager().url(), ?device))]
async fn update(pool: Pool<prometheus::Manager>, device: Arc<Device>) -> Result<Update> {
    trace!("updating");

    match pool.get().await {
        Ok(conn) => device.update(&conn).await,
        Err(e) => Err(eyre!(e)).wrap_err(format!(
            "retrieving connection for {}",
            pool.manager().url()
        )),
    }
}
