mod args;
mod collector;
mod config;
mod device;
mod init;
mod ui;

pub use crate::args::Args;
use collector::Collector;
use eyre::Result;
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tracing::info;
use ui::App;

fn main() -> Result<()> {
    init::tracing();
    init::eyre()?;
    let args = init::args()?;

    tokio_main(args)
}

#[tokio::main]
async fn tokio_main(args: Args) -> Result<()> {
    let collector = Collector::new(&args)?;

    let mut app = App::new(args.tick_rate, args.frame_rate, collector.subscribe())?;

    let ctrl_c = ctrl_c();
    let mut term = signal(SignalKind::terminate())?;

    tokio::select! {
        result = collector.wait() => { result?; },
        result = app.run() => { result? },
        _ = ctrl_c => {
            info!("shutdown requested")
        },
        _ = term.recv() => {
            info!("shutdown requested")
        }
    }

    Ok(())
}
