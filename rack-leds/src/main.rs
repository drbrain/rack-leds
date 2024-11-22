mod args;
mod collector;
mod column;
mod columns;
mod config;
mod device;
mod devices;
mod http;
mod init;
mod layout;
mod png_builder;
mod simulator;
mod ui;
mod update;

use std::sync::{atomic::AtomicBool, Arc};

pub use args::Args;
use collector::Collector;
pub use column::Column;
pub use columns::Columns;
pub use devices::Devices;
use eyre::Result;
pub use http::Http;
pub use layout::Layout;
pub use png_builder::PngBuilder;
use ratatui_tracing::{EventReceiver, Reloadable};
pub use simulator::Simulator;
use tokio::{
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
    sync::mpsc,
    task::{JoinSet, LocalSet},
};
use tracing::{debug, info, instrument};
use ui::{Action, App};
pub use update::Update;

fn main() -> Result<()> {
    let args = init::args()?;

    let (gui_active, event_receiver, reload_handle) = init::tracing(&args);

    init::eyre()?;

    tokio_main(args, gui_active, event_receiver, reload_handle)
}

#[tokio::main]
async fn tokio_main(
    args: Args,
    gui_active: Arc<AtomicBool>,
    event_receiver: EventReceiver,
    reloadable: Reloadable,
) -> Result<()> {
    debug!("args: {:#?}", args);

    let devices: Devices = args.config()?.into();

    let mut tasks = JoinSet::new();

    let updates = if args.simulate {
        let simulator = Simulator::new(&args, &devices)?;
        let updates = simulator.subscribe();

        simulator.run_on(&mut tasks)?;

        updates
    } else {
        let collector = Collector::new(&args, &devices)?;
        let updates = collector.subscribe();

        collector.run_on(&mut tasks)?;

        updates
    };

    let (png_sender, png_receiver) = png_builder::update_channel();
    let http = Http::new(args.server_address, png_receiver, args.period())?;
    tasks.build_task().name("http server").spawn(http.run())?;

    if !args.headless {
        info!("starting TUI");
        let mut app = App::new(
            gui_active,
            event_receiver,
            reloadable,
            args.tick_rate,
            args.frame_rate,
            devices.columns().clone(),
            updates,
            png_sender,
        )?;

        app.run().await?;

        wait_for_sigint(&mut tasks, Some(app.action_tx()))?;
        wait_for_sigterm(&mut tasks, Some(app.action_tx()))?;

        let local_set = LocalSet::new();

        tasks
            .build_task()
            .name("app")
            .spawn_local_on(async move { app.run().await }, &local_set)?;

        local_set
            .run_until(async { first_termination(tasks).await })
            .await?;
    } else {
        info!("starting headless");
        wait_for_sigint(&mut tasks, None)?;
        wait_for_sigterm(&mut tasks, None)?;
        first_termination(tasks).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
fn wait_for_sigint(
    tasks: &mut JoinSet<Result<()>>,
    sender: Option<mpsc::UnboundedSender<Action>>,
) -> Result<()> {
    tasks.build_task().name("SIGINT").spawn(async move {
        loop {
            ctrl_c().await?;

            info!("shutdown requested");

            if let Some(ref sender) = sender {
                sender.send(Action::Quit).ok();
            } else {
                break;
            };
        }

        Ok(())
    })?;

    Ok(())
}

#[instrument(skip_all)]
fn wait_for_sigterm(
    tasks: &mut JoinSet<Result<()>>,
    sender: Option<mpsc::UnboundedSender<Action>>,
) -> Result<()> {
    tasks.build_task().name("SIGTERM").spawn(async move {
        loop {
            signal(SignalKind::terminate())?.recv().await;

            info!("shutdown requested");

            if let Some(ref sender) = sender {
                sender.send(Action::Quit).ok();
            } else {
                break;
            };
        }

        Ok(())
    })?;

    Ok(())
}

async fn first_termination(mut tasks: JoinSet<Result<()>>) -> Result<()> {
    if let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}
