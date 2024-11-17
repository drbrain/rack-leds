use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Wrap},
};

use crate::ratatui_tracing::widgets::EventLogState;

pub struct EventLog<'a> {
    block: Block<'a>,
    highlight_style: Style,
    status_live_style: Style,
    status_paused_style: Style,
    title: String,
    title_style: Style,
}

impl<'a> Default for EventLog<'a> {
    fn default() -> Self {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let title = "Log".to_string();
        let highlight_style = Style::default().bg(Color::DarkGray);
        let status_live_style = Style::default().bold();
        let status_paused_style = Style::default().bold().fg(Color::Yellow);
        let title_style = Style::default().bold();

        Self {
            block,
            highlight_style,
            status_live_style,
            status_paused_style,
            title_style,
            title,
        }
    }
}

impl<'a> StatefulWidget for EventLog<'a> {
    type State = EventLogState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let history = state.history();
        let total = state.total();

        let status = if state.is_live() {
            Line::from(format!("{total} events, live")).style(self.status_live_style)
        } else {
            let snapshot_total = history.total();

            let new = if total > snapshot_total {
                let new = total.saturating_sub(snapshot_total);

                format!(", +{new} live")
            } else {
                "".to_string()
            };

            Line::from(format!(
                "event {} / {}{new}",
                history.offset(),
                history.len()
            ))
            .style(self.status_paused_style)
        };

        let block = self
            .block
            .clone()
            .title(Line::from(self.title).style(self.title_style))
            .title(status.right_aligned());

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
