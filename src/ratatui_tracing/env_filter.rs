use std::env;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    filter::{Directive, ParseError},
    reload, EnvFilter, Registry,
};

use crate::ratatui_tracing::Reloadable;

/// A reloadable [`EnvFilter`] that tracks the [`Directive`]s it currently contains.
//
// tracing-subscriber currently doesn't allow access to the directives in use so we track them here
// so we can build a tui wrapper
pub struct EnvFilterResult {
    /// The reloadable [`EnvFilter`] layer
    pub layer: reload::Layer<EnvFilter, Registry>,

    /// A [`Reloadable`] reload layer handle
    pub reloadable: Reloadable,

    /// Directives that were not parseable from the last reload attempt
    pub invalid_directives: Option<Vec<(String, ParseError)>>,
}

/// Create an [`EnvFilter`] that is reloadable
///
/// If the `default` [`Directive`] is `None` the tracing-subscriber default of `error` is used
///
/// If `env_var` is `None` the tracing-subscriber default of `RUST_LOG` is used.  Any unparseable
/// directives in the env variable are stored in [`EnvFilterResult::invalid_directives`]
pub fn env_filter(default: Option<Directive>, env_var: Option<String>) -> EnvFilterResult {
    let default = default.unwrap_or(LevelFilter::ERROR.into());
    let env_var = env_var.as_deref().unwrap_or(EnvFilter::DEFAULT_ENV);
    let filter = env::var(env_var).unwrap_or_default();

    let env_filter = EnvFilter::builder()
        .with_default_directive(default.clone())
        .parse_lossy("");

    if filter.is_empty() {
        let (layer, reload_handle) = reload::Layer::new(env_filter);

        return EnvFilterResult {
            layer,
            reloadable: Reloadable::new(reload_handle, default, vec![]),
            invalid_directives: None,
        };
    }

    let mut directives = vec![];
    let mut invalid_directives = vec![];

    filter
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<Directive>().map_err(|e| (s.to_string(), e)))
        .for_each(|r| match r {
            Ok(directive) => directives.push(directive),
            Err(invalid) => invalid_directives.push(invalid),
        });

    let filter = directives.iter().fold(env_filter, |filter, directive| {
        filter.add_directive(directive.clone())
    });

    let (layer, reload_handle) = reload::Layer::new(filter);

    let reloadable = Reloadable::new(reload_handle, default, directives);

    let invalid_directives = if invalid_directives.is_empty() {
        None
    } else {
        Some(invalid_directives)
    };

    EnvFilterResult {
        layer,
        reloadable,
        invalid_directives,
    }
}
