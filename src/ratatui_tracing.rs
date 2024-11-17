mod event;
mod history;
mod reloadable;
mod scope;
mod to_scope_visitor;
pub mod widgets;

use std::{env, time::Instant};

pub use event::Event;
use eyre::Result;
pub use reloadable::Reloadable;
pub use scope::Scope;
pub use to_scope_visitor::ToScopeVisitor;
use tokio::sync::broadcast::{
    self,
    error::{RecvError, TryRecvError},
};
use tracing::{
    level_filters::LevelFilter,
    span::{Attributes, Id, Record},
    Subscriber,
};
use tracing_subscriber::{
    filter::{Directive, ParseError},
    layer::Context,
    registry::LookupSpan,
    reload, EnvFilter, Layer, Registry,
};

pub type ReloadHandle = reload::Handle<EnvFilter, Registry>;

pub struct EventReceiver {
    pub epoch: Instant,
    pub channel: broadcast::Receiver<Event>,
}

impl EventReceiver {
    pub async fn recv(&mut self) -> Result<Event, RecvError> {
        self.channel.recv().await
    }

    pub fn resubscribe(&self) -> Self {
        EventReceiver {
            epoch: self.epoch,
            channel: self.channel.resubscribe(),
        }
    }

    pub fn try_recv(&mut self) -> Result<Event, TryRecvError> {
        self.channel.try_recv()
    }
}

pub struct RatatuiTracing {
    sender: broadcast::Sender<Event>,
    epoch: Instant,
}

impl RatatuiTracing {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);

        Self {
            sender,
            epoch: Instant::now(),
        }
    }

    pub fn subscribe(&self) -> EventReceiver {
        EventReceiver {
            epoch: self.epoch,
            channel: self.sender.subscribe(),
        }
    }
}

impl Default for RatatuiTracing {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RatatuiTracing
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, context: Context<'_, S>) {
        let log_line = Event::new(event, &context);

        self.sender.send(log_line).ok();
    }

    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let span = context
            .span(id)
            .expect("Span not found, this is a tracing bug");
        let mut extensions = span.extensions_mut();

        if extensions.get_mut::<Scope>().is_none() {
            let name = span.name().to_string();

            let mut visitor = ToScopeVisitor::default();
            attributes.values().record(&mut visitor);

            extensions.insert(visitor.finish(name));
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        let span = context
            .span(id)
            .expect("Span not found, this is a tracing bug");
        let name = span.name().to_string();

        let mut extensions = span.extensions_mut();

        let mut visitor = ToScopeVisitor::default();
        values.record(&mut visitor);
        let scope = visitor.finish(name);

        if let Some(existing) = extensions.get_mut::<Scope>() {
            existing.extend(scope);
        } else {
            extensions.insert(scope);
        }
    }
}

pub struct EnvFilterResult {
    pub layer: reload::Layer<EnvFilter, Registry>,
    pub reloadable: Reloadable,
    pub invalid_directives: Option<Vec<(String, ParseError)>>,
}

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
