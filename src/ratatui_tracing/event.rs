use std::{collections::HashMap, time::Instant};

use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use time::OffsetDateTime;
use tracing::{Level, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan};

use crate::ratatui_tracing::{FormatInner, Scope, ToScopeVisitor};

use super::format::ScopeDisplay;

#[derive(Clone)]
pub struct Event {
    recorded: Instant,
    recorded_date_time: OffsetDateTime,
    scopes: Vec<Scope>,
    target: String,
    level: Level,
    fields: HashMap<&'static str, String>,
}

impl Event {
    pub fn closed() -> Event {
        Self {
            recorded: Instant::now(),
            recorded_date_time: OffsetDateTime::now_utc(),
            scopes: Default::default(),
            target: "tracing event channel closed".into(),
            level: Level::WARN,
            fields: Default::default(),
        }
    }

    pub fn missed(count: u64) -> Self {
        let fields = HashMap::from([("count", format!("{count}"))]);

        Self {
            recorded: Instant::now(),
            recorded_date_time: OffsetDateTime::now_utc(),
            scopes: Default::default(),
            target: "tracing events missed".into(),
            level: Level::WARN,
            fields,
        }
    }

    pub fn new(
        event: &tracing::Event,
        context: &Context<'_, impl Subscriber + for<'a> LookupSpan<'a>>,
    ) -> Self {
        let recorded = Instant::now();
        let recorded_date_time = OffsetDateTime::now_utc();

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
            recorded,
            recorded_date_time,
            scopes,
            target: metadata.target().to_string(),
            level: *metadata.level(),
            fields,
        }
    }

    pub fn message(&self) -> Option<String> {
        self.fields.get("message").cloned()
    }

    pub fn to_line(&self, epoch: Instant, format: FormatInner) -> Line<'_> {
        let mut line = Line::default();

        if let Some(time) = format.time.format(self, epoch, format.local_offset()) {
            line.push_span(Span::styled(time, DIM));
            line.push_span(Span::raw(" "));
        }

        if format.display_level {
            self.add_level(&mut line);
            line.push_span(Span::raw(" "));
        };

        self.add_scopes(&mut line, &format);

        if format.display_target {
            self.add_target(&mut line);
        }

        self.add_message(&mut line);

        self.add_fields(&mut line);

        line
    }

    fn add_fields<'a>(&'a self, line: &mut Line<'a>) {
        self.fields
            .iter()
            .filter(|(k, _)| *k != &"message")
            .sorted_by_cached_key(|(name, _)| *name)
            .for_each(|(name, value)| {
                line.push_span(Span::raw(" "));
                line.push_span(Span::raw(*name));
                line.push_span(Span::raw("="));
                line.push_span(Span::raw(value));
            });
    }

    fn add_level(&self, line: &mut Line<'_>) {
        let level = match self.level {
            Level::ERROR => Span::styled("ERROR", ERROR_STYLE),
            Level::WARN => Span::styled("WARN ", WARN_STYLE),
            Level::INFO => Span::styled("INFO ", INFO_STYLE),
            Level::DEBUG => Span::styled("DEBUG", DEBUG_STYLE),
            Level::TRACE => Span::styled("TRACE", TRACE_STYLE),
        };

        line.push_span(level);
    }

    fn add_message(&self, line: &mut Line<'_>) {
        if let Some(message) = self.message() {
            line.push_span(Span::raw(" "));
            line.push_span(Span::raw(message));
        }
    }

    fn add_scopes<'a>(&'a self, line: &mut Line<'a>, format: &FormatInner) {
        match format.display_scope {
            ScopeDisplay::All => {
                self.scopes.iter().for_each(|scope| {
                    add_scope(line, scope, format);
                });
                line.push_span(Span::raw(" "));
            }
            ScopeDisplay::Last => {
                if let Some(scope) = self.scopes.last() {
                    add_scope(line, scope, format);
                }
                line.push_span(Span::raw(" "));
            }
            ScopeDisplay::None => (),
        }
    }

    fn add_target(&self, line: &mut Line<'_>) {
        line.push_span(Span::styled(self.target.clone(), DIM));
        line.push_span(Span::styled(":", DIM));
    }

    pub fn recorded(&self) -> Instant {
        self.recorded
    }

    pub fn recorded_date_time(&self) -> OffsetDateTime {
        self.recorded_date_time
    }
}

fn add_scope<'a>(line: &mut Line<'a>, scope: &'a Scope, format: &FormatInner) {
    line.push_span(Span::styled(scope.name(), BOLD));

    if format.display_scope_fields {
        line.push_span(Span::styled("{", BOLD));

        scope
            .fields()
            .sorted_by_cached_key(|(field, _)| *field)
            .enumerate()
            .for_each(|(index, (field, value))| {
                line.push_span(Span::styled(*field, ITALIC));
                line.push_span(Span::styled("=", DIM));
                line.push_span(Span::raw(value));
                if index != scope.len() - 1 {
                    line.push_span(Span::raw(" "));
                }
            });

        line.push_span(Span::styled("}", BOLD));
    }

    line.push_span(Span::raw(":"));
}

const ERROR_STYLE: Style = Style::new().fg(Color::Red);
const WARN_STYLE: Style = Style::new().fg(Color::Yellow);
const INFO_STYLE: Style = Style::new().fg(Color::White);
const DEBUG_STYLE: Style = Style::new().fg(Color::Blue);
const TRACE_STYLE: Style = Style::new().fg(Color::Cyan);

const DIM: Style = Style::new().add_modifier(Modifier::DIM);
const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
const ITALIC: Style = Style::new().add_modifier(Modifier::ITALIC);
