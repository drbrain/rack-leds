use std::time::Instant;

use time::{format_description::well_known, OffsetDateTime, UtcOffset};

use crate::Event;

#[derive(Clone, Copy, Default, strum::IntoStaticStr)]
pub enum TimeFormat {
    #[strum(serialize = "Local RFC3339")]
    Rfc3339Local,
    #[strum(serialize = "UTC RFC3339")]
    Rfc3339Utc,
    /// Hide time
    #[strum(serialize = "Hide")]
    None,
    /// Time since the process started
    #[default]
    #[strum(serialize = "Process Uptime")]
    Uptime,
}

impl TimeFormat {
    pub fn next(&self, local_offset: Option<UtcOffset>) -> Self {
        match self {
            TimeFormat::Rfc3339Local => TimeFormat::Rfc3339Utc,
            TimeFormat::Rfc3339Utc => TimeFormat::Uptime,
            TimeFormat::Uptime => TimeFormat::None,
            TimeFormat::None => {
                if local_offset.is_some() {
                    TimeFormat::Rfc3339Local
                } else {
                    TimeFormat::Rfc3339Utc
                }
            }
        }
    }

    pub fn format(
        &self,
        event: &Event,
        epoch: Instant,
        local_offset: Option<UtcOffset>,
    ) -> Option<String> {
        match (self, local_offset) {
            (TimeFormat::Rfc3339Local, Some(local_offset)) => Some(format_rfc3339(
                event.recorded_date_time(),
                Some(local_offset),
            )),
            (TimeFormat::Rfc3339Local, None) | (TimeFormat::Rfc3339Utc, _) => {
                Some(format_rfc3339(event.recorded_date_time(), None))
            }
            (TimeFormat::None, _) => None,
            (TimeFormat::Uptime, _) => {
                let elapsed = event.recorded().saturating_duration_since(epoch);

                Some(format!("{:.6}", elapsed.as_secs_f64()))
            }
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
