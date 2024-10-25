mod args;
mod collector;
mod config;
mod device;
mod http;
mod init;
mod layout;
mod png_builder;
mod ratatui_tracing;
mod ui;
mod update;

use std::sync::{atomic::AtomicBool, Arc};

pub use args::Args;
use collector::Collector;
use eyre::Result;
pub use http::Http;
pub use layout::Layout;
pub use png_builder::PngBuilder;
use ratatui_tracing::EventReceiver;
pub use ratatui_tracing::RatatuiTracing;
use tokio::{
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
    sync::mpsc,
    task::{JoinSet, LocalSet},
};
use tracing::{info, instrument};
use ui::{Action, App};
pub use update::Update;

fn main() -> Result<()> {
    let args = init::args()?;
    let (gui_active, event_receiver) = init::tracing(&args);
    init::eyre()?;

    tokio_main(args, gui_active, event_receiver)
}

#[tokio::main]
async fn tokio_main(
    args: Args,
    gui_active: Arc<AtomicBool>,
    event_receiver: EventReceiver,
) -> Result<()> {
    let mut tasks = JoinSet::new();

    let collector = Collector::new(&args)?;
    let updates = collector.subscribe();
    tasks
        .build_task()
        .name("collector outer")
        .spawn(async move { collector.wait().await })?;

    let (png_sender, png_receiver) = png_builder::update_channel();
    let http = Http::new(args.server_address, png_receiver, args.period())?;
    tasks.build_task().name("http server").spawn(http.run())?;

    if !args.headless {
        info!("starting TUI");
        let mut app = App::new(
            gui_active,
            event_receiver,
            args.tick_rate,
            args.frame_rate,
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
            };
        }
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
            };
        }
    })?;

    Ok(())
}

async fn first_termination(mut tasks: JoinSet<Result<()>>) -> Result<()> {
    if let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}
