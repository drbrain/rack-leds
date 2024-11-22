mod env_filter;
mod event;
mod event_receiver;
mod history;
mod reloadable;
mod scope;
mod to_scope_visitor;
pub mod widgets;

use std::time::Instant;

pub use env_filter::{env_filter, EnvFilterResult};
pub use event::Event;
pub use event_receiver::EventReceiver;
pub use history::History;
pub use reloadable::Reloadable;
pub use scope::Scope;
pub use to_scope_visitor::ToScopeVisitor;
use tokio::sync::broadcast;
use tracing::{
    span::{Attributes, Id, Record},
    Subscriber,
};
use tracing_subscriber::{
    layer::Context, registry::LookupSpan, reload, EnvFilter, Layer, Registry,
};

/// Reloadable [`EnvFilter`] wrapper type
pub type ReloadHandle = reload::Handle<EnvFilter, Registry>;

pub struct RatatuiTracing {
    sender: broadcast::Sender<Event>,
    epoch: Instant,
}

impl RatatuiTracing {
    /// Create a ratatui tracing layer
    ///
    /// Allow `capacity` in-flight events per receiver
    ///
    /// The `epoch` is used to determine process start time for relative time formatting in the log
    pub fn new(capacity: usize, epoch: Instant) -> Self {
        let (sender, _) = broadcast::channel(capacity);

        Self { sender, epoch }
    }

    /// Subscribe to events recorded by this layer
    pub fn subscribe(&self) -> EventReceiver {
        EventReceiver {
            epoch: self.epoch,
            channel: self.sender.subscribe(),
        }
    }
}

impl Default for RatatuiTracing {
    /// A ratatui tracing layer with an epoch of now and storage for 100 in-flight events
    fn default() -> Self {
        Self::new(100, Instant::now())
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
