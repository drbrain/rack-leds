use ratatui::{
    layout::Alignment,
    text::{Text, ToText},
    widgets::Row,
};
use time::UtcOffset;

use crate::{
    ratatui_tracing::format::{ScopeDisplay, TimeFormat},
    LOCAL_OFFSET,
};

#[derive(Clone)]
pub struct FormatInner {
    pub time: TimeFormat,
    pub display_level: bool,
    pub display_scope: ScopeDisplay,
    pub display_scope_fields: bool,
    pub display_target: bool,
    local_offset: UtcOffset,
}

impl FormatInner {
    pub fn as_rows(&self) -> Vec<Row> {
        vec![
            Row::new(vec![
                Text::from("Time").alignment(Alignment::Left),
                self.time.to_text().alignment(Alignment::Right),
            ]),
            Row::new(vec![
                Text::from("Level").alignment(Alignment::Left),
                Text::from(visibility(self.display_level)).alignment(Alignment::Right),
            ]),
            Row::new(vec![
                Text::from("Scope Display").alignment(Alignment::Left),
                self.display_scope.to_text().alignment(Alignment::Right),
            ]),
            Row::new(vec![
                Text::from("Scope Fields").alignment(Alignment::Left),
                Text::from(visibility(self.display_scope_fields)).alignment(Alignment::Right),
            ]),
            Row::new(vec![
                Text::from("Target").alignment(Alignment::Left),
                Text::from(visibility(self.display_target)).alignment(Alignment::Right),
            ]),
        ]
    }

    pub fn local_offset(&self) -> UtcOffset {
        self.local_offset
    }
}

impl Default for FormatInner {
    fn default() -> Self {
        let local_offset = *LOCAL_OFFSET.get().expect("init::local_offset() not called");

        Self {
            time: Default::default(),
            display_level: true,
            display_scope: Default::default(),
            display_scope_fields: true,
            display_target: true,
            local_offset,
        }
    }
}

fn visibility(visible: bool) -> &'static str {
    if visible {
        "Show"
    } else {
        "Hide"
    }
}
