use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Wrap},
};

use crate::ratatui_tracing::widgets::EventLogState;

pub struct EventLog<'a> {
    block: Block<'a>,
    highlight_style: Style,
    title: String,
}

impl<'a> Default for EventLog<'a> {
    fn default() -> Self {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let title = "Log".to_string();
        let highlight_style = Style::default().bg(Color::DarkGray);

        Self {
            block,
            highlight_style,
            title,
        }
    }
}

impl<'a> StatefulWidget for EventLog<'a> {
    type State = EventLogState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let history = state.history();

        let status = if state.is_live() {
            "Live".into()
        } else {
            let live = state.live().len();

            format!("{} / {} ({live} live)", history.offset(), history.len())
        };

        let status = Line::from(status).right_aligned();

        let block = self.block.clone().title(self.title).title(status);

        let log_area = block.inner(area);
        block.render(area, buf);

        let mut current_height = 0;

        let events = state.history().events().map(|(i, event)| {
            let line = event.to_line(state.epoch(), &state.format);

            let paragraph = Paragraph::new(line).wrap(Wrap { trim: true });
            (i, paragraph)
        });

        let selected = state.history().selected;

        for (i, event) in events {
            let height = event.line_count(log_area.width) as u16;
            current_height += height;

            if current_height >= log_area.height {
                break;
            }

            let event_area = Rect {
                x: log_area.left(),
                y: log_area.bottom() - current_height,
                width: log_area.width,
                height,
            };

            event.render(event_area, buf);

            if selected.map_or(false, |s| s == i) {
                buf.set_style(event_area, self.highlight_style);
            }
        }
    }
}
