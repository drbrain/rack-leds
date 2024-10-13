use std::collections::HashMap;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, ToLine},
};
use tokio::sync::broadcast;
use tracing::{
    field::{Field, Visit},
    Event, Level, Subscriber,
};
use tracing_subscriber::{layer::Context, Layer};

#[derive(Clone)]
pub struct LogLine {
    name: &'static str,
    target: String,
    level: Level,
    file: Option<String>,
    line: Option<u32>,
    module_path: Option<String>,
    fields: HashMap<&'static str, String>,
}

impl LogLine {
    pub fn missed(count: u64) -> Self {
        let fields = HashMap::from([("count", format!("{count}"))]);

        Self {
            name: "synthetic",
            target: "missed".into(),
            level: Level::WARN,
            file: None,
            line: None,
            module_path: None,
            fields,
        }
    }
}

impl ToLine for LogLine {
    fn to_line(&self) -> Line<'_> {
        let mut line = Line::default();

        let level = match self.level {
            Level::ERROR => Span::styled("ERROR", Style::default().fg(Color::Red)),
            Level::WARN => Span::styled("WARN ", Style::default().fg(Color::Yellow)),
            Level::INFO => Span::styled("INFO ", Style::default().fg(Color::White)),
            Level::DEBUG => Span::styled("DEBUG", Style::default().fg(Color::Blue)),
            Level::TRACE => Span::styled("TRACE", Style::default().fg(Color::Cyan)),
        };

        line.push_span(level);
        line.push_span(Span::raw(" "));

        line.push_span(Span::styled(
            self.target.clone(),
            Style::default().add_modifier(Modifier::ITALIC),
        ));
        line.push_span(Span::raw(" "));

        for (name, value) in self.fields.iter() {
            line.push_span(Span::raw(*name));
            line.push_span(Span::raw("="));
            line.push_span(Span::raw(value));
            line.push_span(Span::raw(" "));
        }

        line
    }
}

impl From<&Event<'_>> for LogLine {
    fn from(event: &Event) -> Self {
        let mut visitor = ToFieldsVisitor::default();

        event.record(&mut visitor);

        let fields = visitor.fields();

        Self {
            name: event.metadata().name(),
            target: event.metadata().target().to_string(),
            level: *event.metadata().level(),
            file: event.metadata().file().map(|f| f.to_string()),
            line: event.metadata().line(),
            module_path: event.metadata().module_path().map(|p| p.to_string()),
            fields,
        }
    }
}

#[derive(Default)]
struct ToFieldsVisitor {
    fields: HashMap<&'static str, String>,
}

impl ToFieldsVisitor {
    fn fields(self) -> HashMap<&'static str, String> {
        self.fields
    }
}

impl Visit for ToFieldsVisitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name(), format!("{value:?}"));
    }
}

pub struct TuiTracing {
    sender: broadcast::Sender<LogLine>,
}

impl TuiTracing {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);

        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LogLine> {
        self.sender.subscribe()
    }
}

impl<S> Layer<S> for TuiTracing
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let log_line = event.into();

        self.sender.send(log_line).ok();
    }
}
