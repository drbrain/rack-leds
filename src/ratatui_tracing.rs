use std::collections::HashMap;

use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, ToLine},
};
use tokio::sync::broadcast;
use tracing::{
    field::{Field, Visit},
    span::{Attributes, Id, Record},
    Event, Level, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Clone)]
pub struct LogLine {
    scopes: Vec<Scope>,
    target: String,
    level: Level,
    fields: HashMap<&'static str, String>,
}

impl LogLine {
    pub fn missed(count: u64) -> Self {
        let fields = HashMap::from([("count", format!("{count}"))]);

        Self {
            scopes: vec![],
            target: "missed".into(),
            level: Level::WARN,
            fields,
        }
    }

    pub fn new(
        event: &Event,
        context: &Context<'_, impl Subscriber + for<'a> LookupSpan<'a>>,
    ) -> Self {
        let mut visitor = ToFieldsVisitor::default();

        event.record(&mut visitor);

        let fields = visitor.fields();

        let metadata = event.metadata();

        let mut scopes = vec![];
        if let Some(mut span) = context.event_span(event) {
            loop {
                {
                    let ext = span.extensions();
                    if let Some(scope) = ext.get::<Scope>() {
                        scopes.push(scope.clone());
                    }
                }

                span = if let Some(parent) = span.parent() {
                    parent
                } else {
                    break;
                }
            }
        }

        scopes.reverse();

        Self {
            scopes,
            target: metadata.target().to_string(),
            level: *metadata.level(),
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

        for (index, scope) in self.scopes.iter().enumerate() {
            line.push_span(Span::styled(&scope.name, Style::default().bold()));
            line.push_span(Span::styled("{", Style::default().bold()));
            for (index, (field, value)) in scope.fields.iter().enumerate() {
                line.push_span(Span::styled(*field, Style::default().italic()));
                line.push_span(Span::styled("=", Style::default().dim()));
                line.push_span(Span::raw(value));
                if index != scope.fields.len() - 1 {
                    line.push_span(Span::raw(" "));
                }
            }
            line.push_span(Span::styled("}", Style::default().bold()));
            if index == self.scopes.len() - 1 {
                line.push_span(Span::raw(" "));
            } else {
                line.push_span(Span::raw(":"));
            }
        }

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

#[derive(Clone, Debug)]
struct Scope {
    name: String,
    fields: HashMap<&'static str, String>,
}

impl Scope {
    fn new(name: String, fields: HashMap<&'static str, String>) -> Self {
        Self { name, fields }
    }

    fn extend(&mut self, fields: HashMap<&'static str, String>) {
        self.fields.extend(fields);
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
        self.fields.insert(field.name(), value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(field.name(), format!("{value}"));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name(), format!("{value:?}"));
    }
}

pub struct RatatuiTracing {
    sender: broadcast::Sender<LogLine>,
}

impl RatatuiTracing {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);

        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LogLine> {
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

            let mut visitor = ToFieldsVisitor::default();
            attributes.values().record(&mut visitor);
            let fields = visitor.fields();

            let scope = Scope::new(name, fields);

            extensions.insert(scope);
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        let span = context
            .span(id)
            .expect("Span not found, this is a tracing bug");

        let mut extensions = span.extensions_mut();

        let mut visitor = ToFieldsVisitor::default();
        values.record(&mut visitor);
        let fields = visitor.fields();

        if let Some(scope) = extensions.get_mut::<Scope>() {
            scope.extend(fields);
        } else {
            let name = span.name().to_string();

            let scope = Scope::new(name, fields);

            extensions.insert(scope);
        }
    }
}
