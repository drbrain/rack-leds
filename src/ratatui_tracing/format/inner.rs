use ratatui::{
    layout::Alignment,
    text::{Text, ToText},
    widgets::Row,
};
use time::UtcOffset;

use crate::{ratatui_tracing::format::TimeFormat, LOCAL_OFFSET};

#[derive(Clone, Copy)]
pub struct FormatInner {
    pub time: TimeFormat,
    pub display_level: bool,
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
