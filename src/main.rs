mod args;
mod collector;
mod config;
mod device;
mod init;
mod layout;
mod ui;
mod update;

pub use crate::args::Args;
pub use crate::layout::Layout;
pub use crate::update::Update;
use collector::Collector;
use eyre::Result;
use tokio::signal::{
    ctrl_c,
    unix::{signal, SignalKind},
};
use tokio::task::JoinSet;
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
    let mut tasks = JoinSet::new();

    let collector = Collector::new(&args)?;
    let updates = collector.subscribe();
    tasks.spawn(async move { collector.wait().await });

    tasks.spawn(async {
        ctrl_c().await?;

        info!("shutdown requested");

        Ok(())
    });

    tasks.spawn(async {
        signal(SignalKind::terminate())?.recv().await;

        info!("shutdown requested");

        Ok(())
    });

    if args.headless {
        if let Some(result) = tasks.join_next().await {
            result??;
        }
    } else {
        let mut app = App::new(args.tick_rate, args.frame_rate, updates)?;
        app.run().await?;
    }

    tasks.abort_all();

    Ok(())
}
