use std::sync::{atomic::AtomicBool, Arc};

use crate::{ratatui_tracing::EventReceiver, Args, RatatuiTracing};
use clap::Parser;
use color_eyre::config::HookBuilder;
use eyre::Result;
use std::{io::IsTerminal, sync::atomic::Ordering};
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::{filter_fn, Directive, LevelFilter},
    fmt::{self, time::OffsetTime},
    prelude::*,
    EnvFilter, Layer,
};

pub(crate) fn args() -> Result<Args> {
    Ok(Args::parse())
}

pub(crate) fn eyre() -> Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::new()
        .capture_span_trace_by_default(true)
        .display_env_section(false)
        .display_location_section(true)
        .panic_section("Oh no, a bug")
        .into_hooks();

    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        // TODO: Why won't move move this?
        let gui_active = Arc::new(AtomicBool::new(true));

        if let Ok(mut t) = crate::ui::Tui::new(gui_active) {
            if let Err(report) = t.exit() {
                error!(?report, "unable to exit terminal");
            }
        }

        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let metadata = metadata!();
            let file_path = handle_dump(&metadata, panic_info);
            // prints human-panic message
            print_msg(file_path, &metadata)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info)); // prints color-eyre stack trace to stderr
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(libc::EXIT_FAILURE);
    }));

    Ok(())
}

pub(crate) fn tracing(args: &Args) -> (Arc<AtomicBool>, EventReceiver) {
    let (gui_active, reader, log) = log_layer();

    let registry = tracing_subscriber::registry()
        .with(log)
        .with(ErrorLayer::default());

    if args.console {
        registry.with(console_subscriber::spawn()).init();
    } else {
        registry.init();
    };

    (gui_active, reader)
}

/// A layer for logging either to stdout or ratatui depending on which is active
///
/// The layer will be filtered by RUST_LOG if available
fn log_layer() -> (
    Arc<AtomicBool>,
    tokio::sync::broadcast::Receiver<crate::ratatui_tracing::Event>,
    Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>,
) {
    let filter = log_filter();

    let gui_active = Arc::new(AtomicBool::new(true));

    let stdout = stdout_layer(&gui_active);

    let (reader, tui) = ratatui_layer(&gui_active);

    let log = stdout.and_then(tui).with_filter(filter).boxed();
    (gui_active, reader, log)
}

/// Create a filter from RUST_LOG
fn log_filter() -> EnvFilter {
    let default_directive: Directive = LevelFilter::INFO.into();

    let result = EnvFilter::builder()
        .with_default_directive(default_directive.clone())
        .from_env();

    let (filter, error) = match result {
        Ok(filter) => (filter, None),
        Err(e) => {
            let filter = EnvFilter::builder()
                .with_default_directive(default_directive)
                .parse_lossy("");

            (filter, Some(e))
        }
    };

    if let Some(error) = error {
        error!(?error, "Invalid RUST_LOG, using default filter \"info\"");
    }
    filter
}

/// Log to stdout when gui_active is false
fn stdout_layer(
    gui_active: &Arc<AtomicBool>,
) -> Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync> {
    let stdout = fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_timer(OffsetTime::local_rfc_3339().expect("could not get local offset!"));

    let stdout_gui_active = gui_active.clone();

    stdout
        .with_filter(filter_fn(move |_| {
            !stdout_gui_active.load(Ordering::Relaxed)
        }))
        .boxed()
}

/// Log to ratatui if gui_active is true
fn ratatui_layer(
    gui_active: &Arc<AtomicBool>,
) -> (
    tokio::sync::broadcast::Receiver<crate::ratatui_tracing::Event>,
    Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>,
) {
    let tui = RatatuiTracing::new();
    let reader = tui.subscribe();
    let tui_gui_active = gui_active.clone();
    let tui = tui
        .with_filter(filter_fn(move |_| tui_gui_active.load(Ordering::Relaxed)))
        .boxed();
    (reader, tui)
}
