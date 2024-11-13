use std::time::Instant;

use ratatui::text::{Text, ToText};
use time::{format_description::well_known, OffsetDateTime, UtcOffset};

use crate::ratatui_tracing::Event;

#[derive(Clone, Copy, Default)]
pub enum TimeFormat {
    Rfc3339Local,
    Rfc3339Utc,
    /// Hide time
    None,
    /// Time since the process started
    #[default]
    Uptime,
}

impl TimeFormat {
    pub fn next(&self) -> Self {
        match self {
            TimeFormat::Rfc3339Local => TimeFormat::Rfc3339Utc,
            TimeFormat::Rfc3339Utc => TimeFormat::Uptime,
            TimeFormat::Uptime => TimeFormat::None,
            TimeFormat::None => TimeFormat::Rfc3339Local,
        }
    }

    pub fn format(&self, event: &Event, epoch: Instant, local_offset: UtcOffset) -> Option<String> {
        match self {
            TimeFormat::Rfc3339Local => Some(format_rfc3339(
                event.recorded_date_time(),
                Some(local_offset),
            )),
            TimeFormat::Rfc3339Utc => Some(format_rfc3339(event.recorded_date_time(), None)),
            TimeFormat::Uptime => {
                let elapsed = event.recorded().saturating_duration_since(epoch);

                Some(format!("{:.6}", elapsed.as_secs_f64()))
            }
            TimeFormat::None => None,
        }
    }
}

fn format_rfc3339(recorded: OffsetDateTime, offset: Option<UtcOffset>) -> String {
    let recorded = offset
        .map(|offset| recorded.to_offset(offset))
        .unwrap_or(recorded);

    recorded
        .format(&well_known::Rfc3339)
        .unwrap_or("<unknown>".to_string())
}

impl ToText for TimeFormat {
    fn to_text(&self) -> Text<'_> {
        let text = match self {
            TimeFormat::Rfc3339Local => "Local RFC3339",
            TimeFormat::Rfc3339Utc => "UTC RFC3339",
            TimeFormat::None => "Hide",
            TimeFormat::Uptime => "Process Uptime",
        };

        Text::from(text)
    }
}
