use std::{
    collections::HashMap,
    fmt::Debug,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use eyre::Result;
use rand::{
    distributions::Uniform, prelude::Distribution, rngs::SmallRng, seq::SliceRandom, Rng,
    SeedableRng,
};
use tokio::{sync::watch, task::JoinSet, time};
use tracing::{debug, info, instrument};

use crate::{
    collector::{UpdateReceiver, UpdateSender},
    device::{Device, Id},
    update, Args, Devices, Layout, Update,
};

const PORTS: [usize; 4] = [5, 8, 10, 18];
const DISABLED_THRESHOLD: f64 = 0.1;

static TRAFFIC_HIGH: u64 = 1000;

pub struct Simulator {
    devices: Vec<Simulated>,
    period: Duration,
    rng: SmallRng,
    update_sender: UpdateSender,
}

impl Simulator {
    pub fn new(args: &Args, devices: &Devices) -> Result<Self> {
        let (update_sender, _) = watch::channel((HashMap::default(), UNIX_EPOCH));

        let mut rng = SmallRng::from_entropy();
        let traffic = Uniform::new(0, TRAFFIC_HIGH);

        let mut devices: Vec<_> = devices
            .devices()
            .values()
            .map(|device| match device.as_ref() {
                Device::Switch { id, .. } => {
                    let ports = PORTS.choose(&mut rng).unwrap();
                    let mut weights = Vec::with_capacity(*ports);

                    for _ in 0..*ports {
                        if rng.gen::<f64>() < DISABLED_THRESHOLD {
                            weights.push(Uniform::new_inclusive(0, 0));
                        } else {
                            let low = traffic.sample(&mut rng);
                            let high = 1 + low + traffic.sample(&mut rng);
                            weights.push(Uniform::new(low, high));
                        }
                    }

                    Simulated::Switch {
                        id: *id,
                        ports: *ports,
                        weights,
                    }
                }
            })
            .collect();

        devices.sort_by_cached_key(|simulated| simulated.id());

        debug!("devices: {:#?}", devices);

        Ok(Self {
            devices,
            period: args.period(),
            rng,
            update_sender,
        })
    }

    #[instrument(name = "simulator", skip_all)]
    pub async fn run(self) -> Result<()> {
        let Self {
            devices,
            period,
            mut rng,
            update_sender,
        } = self;

        let mut interval = time::interval(period);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        info!(period = ?interval.period(), "started");

        let device_count = devices.len();

        loop {
            interval.tick().await;

            debug!(count = ?device_count, "simulating device updates");

            let mut updates = HashMap::with_capacity(device_count);
            for device in devices.iter() {
                let update = device.simulate(&mut rng);

                updates.insert(device.id(), update);
            }

            update_sender.send_replace((updates, SystemTime::now()));
        }
    }

    pub fn run_on(self, join_set: &mut JoinSet<Result<()>>) -> Result<()> {
        join_set
            .build_task()
            .name("simulator")
            .spawn(async move { self.run().await })?;

        Ok(())
    }

    pub fn subscribe(&self) -> UpdateReceiver {
        self.update_sender.subscribe()
    }
}

#[derive(Clone)]
enum Simulated {
    Switch {
        id: Id,
        ports: usize,
        weights: Vec<Uniform<u64>>,
    },
}

impl Simulated {
    pub fn id(&self) -> Id {
        match self {
            Simulated::Switch { id, .. } => *id,
        }
    }

    pub fn simulate(&self, rng: &mut SmallRng) -> Update {
        match self {
            Simulated::Switch { id, ports, weights } => {
                let receive = weights.iter().map(|weight| weight.sample(rng)).collect();

                let transmit = weights.iter().map(|weight| weight.sample(rng)).collect();

                let poe = weights.iter().map(|weight| weight.sample(rng)).collect();

                let device = update::Switch::new(receive, transmit, poe);
                let layout = Layout::simulate(*ports);

                Update::Switch {
                    id: *id,
                    device,
                    layout,
                }
            }
        }
    }
}

impl Debug for Simulated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Switch { id, ports, weights } => f
                .debug_struct("Switch")
                .field("id", id)
                .field("ports", ports)
                .field("weights", weights)
                .finish(),
        }
    }
}
