use clap::Parser;
use eyre::Result;
use tracing::error;

use crate::Args;

pub(crate) fn args() -> Result<Args> {
    Ok(Args::parse())
}

pub(crate) fn eyre() -> Result<()> {
    use color_eyre::config::HookBuilder;

    let (panic_hook, eyre_hook) = HookBuilder::new()
        .capture_span_trace_by_default(true)
        .display_env_section(false)
        .display_location_section(true)
        .panic_section("Oh no, a bug")
        .into_hooks();

    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        if let Ok(mut t) = crate::ui::Tui::new() {
            if let Err(r) = t.exit() {
                error!("Unable to exit Terminal: {:?}", r);
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

pub(crate) fn tracing() {
    use std::io::IsTerminal;

    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{
        filter::{Directive, LevelFilter},
        fmt::{self, time::OffsetTime},
        EnvFilter,
    };

    let fmt = fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_timer(OffsetTime::local_rfc_3339().expect("could not get local offset!"));

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

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(ErrorLayer::default())
        .init();

    if let Some(error) = error {
        error!(?error, "Invalid RUST_LOG, using default filter \"info\"");
    }
}
