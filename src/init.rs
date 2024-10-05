use clap::Parser;
use eyre::Result;
use tracing::error;

use crate::Args;

pub(crate) fn args() -> Result<Args> {
    Ok(Args::parse())
}

pub(crate) fn eyre() -> Result<()> {
    use color_eyre::config::HookBuilder;

    HookBuilder::new()
        .capture_span_trace_by_default(true)
        .display_env_section(false)
        .display_location_section(true)
        .install()
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
