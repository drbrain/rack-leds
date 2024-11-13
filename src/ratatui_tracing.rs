mod event;
mod event_log;
mod format;
mod scope;
mod to_scope_visitor;

use std::time::Instant;

pub use event::Event;
pub use event_log::EventLog;
pub use format::{Format, FormatInner};
pub use scope::Scope;
pub use to_scope_visitor::ToScopeVisitor;
use tokio::sync::broadcast::{
    self,
    error::{RecvError, TryRecvError},
};
use tracing::{
    span::{Attributes, Id, Record},
    Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

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
