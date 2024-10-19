use crate::ratatui_tracing::{Scope, ToScopeVisitor};
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, ToLine},
};
use std::collections::HashMap;
use tracing::{Level, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan};

#[derive(Clone)]
pub struct Event {
    scopes: Vec<Scope>,
    target: String,
    level: Level,
    fields: HashMap<&'static str, String>,
}

impl Event {
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
        event: &tracing::Event,
        context: &Context<'_, impl Subscriber + for<'a> LookupSpan<'a>>,
    ) -> Self {
        let mut visitor = ToScopeVisitor::default();

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

impl ToLine for Event {
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
            line.push_span(Span::styled(scope.name(), Style::default().bold()));
            line.push_span(Span::styled("{", Style::default().bold()));
            for (index, (field, value)) in scope.fields().enumerate() {
                line.push_span(Span::styled(*field, Style::default().italic()));
                line.push_span(Span::styled("=", Style::default().dim()));
                line.push_span(Span::raw(value));
                if index != scope.len() - 1 {
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
