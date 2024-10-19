mod log_line;
mod scope;
mod to_scope_visitor;

pub use log_line::LogLine;
pub use scope::Scope;
pub use to_scope_visitor::ToScopeVisitor;
use tokio::sync::broadcast;
use tracing::{
    span::{Attributes, Id, Record},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub type EventReceiver = broadcast::Receiver<LogLine>;

pub struct RatatuiTracing {
    sender: broadcast::Sender<LogLine>,
}

impl RatatuiTracing {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);

        Self { sender }
    }

    pub fn subscribe(&self) -> EventReceiver {
        self.sender.subscribe()
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
    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let log_line = LogLine::new(event, &context);

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
