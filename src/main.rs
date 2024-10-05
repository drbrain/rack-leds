mod args;
mod diff;
mod init;
mod prometheus;
mod update;

pub use crate::args::Args;
pub use crate::diff::Diff;
pub use crate::update::Update;
use eyre::{Context, Result};
use prometheus::Prometheus;
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tracing::info;

fn main() -> Result<()> {
    init::tracing();
    init::eyre()?;
    let args = init::args()?;

    tokio_main(args)
}

#[tokio::main]
async fn tokio_main(args: Args) -> Result<()> {
    let prometheus = Prometheus::new(&args.source, args.period(), args.timeout())?;

    let (_update, collector) = prometheus.collect();

    let ctrl_c = ctrl_c();
    let mut term = signal(SignalKind::terminate())?;

    tokio::select! {
        result = collector => { result.wrap_err("collector error")??; },
        _ = ctrl_c => {
            info!("shutdown requested")
        },
        _ = term.recv() => {
            info!("shutdown requested")
        }
    }

    Ok(())
}
