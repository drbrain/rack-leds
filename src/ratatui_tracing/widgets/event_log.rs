use std::convert::Into;

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Wrap},
};

use crate::ratatui_tracing::{widgets::EventLogState, Event};

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

        let event_title = if state.is_live() {
            Line::from(format!("{total} events")).style(self.status_live_style)
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

        let status_title = if state.is_live() {
            Line::from("Live").style(self.status_live_style)
        } else {
            Line::from("PAUSED").style(self.status_paused_style)
        };

        let block = self
            .block
            .clone()
            .title(Line::from(self.title.clone()).style(self.title_style))
            .title(status_title.centered())
            .title(event_title.right_aligned());

        let log_area = block.inner(area);
        block.render(area, buf);

        let (log_area, scroll_area_vertical) = state.scroll_area_vertical(log_area);

        let mut events = state.history().events();

        let selected = state.history().selected;
        let epoch = state.epoch();

        if state.is_detail() {
            let selected = selected.unwrap_or(0);

            let event = events
                .find_map(|(i, event)| {
                    if i == selected {
                        Some(event.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| Event::dropped(selected, total).into());

            event.to_pretty(epoch, &state.format).render(log_area, buf);
        } else {
            let events = events.map(|(i, event)| (i, event.to_line(epoch, &state.format)));

            let (log_area, scroll_area_horizontal) = state.scroll_area_horizontal(log_area);

            let mut current_height = 0;
            let mut max_width = 0;
            let mut visible = 0;

            for (i, event) in events {
                visible += 1;
                max_width = max_width.max(event.width());

                let event = Paragraph::new(event);

                let event = if state.format.wrap() {
                    event.wrap(Wrap { trim: false })
                } else {
                    event.scroll((0, state.horizontal_offset))
                };

                let mut truncate = false;
                let remaining_height = log_area.height.saturating_sub(current_height);
                let height = event.line_count(log_area.width) as u16;

                let height = if remaining_height == 0 {
                    break;
                } else if remaining_height < height {
                    truncate = true;
                    current_height += remaining_height;
                    remaining_height
                } else {
                    current_height += height;
                    height
                };

                let event_area = Rect {
                    x: log_area.left(),
                    y: log_area.bottom() - current_height,
                    width: log_area.width,
                    height,
                };

                event.render(event_area, buf);

                if truncate {
                    add_truncate(event_area, buf);

                    break;
                }

                if selected.map_or(false, |s| s == i) {
                    buf.set_style(event_area, self.highlight_style);
                }
            }

            if let Some(scroll_area) = scroll_area_vertical {
                let mut scroll_bar_state =
                    Into::<ScrollbarState>::into(state.history()).viewport_content_length(visible);

                Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                    .symbols(ratatui::symbols::scrollbar::VERTICAL)
                    .render(scroll_area, buf, &mut scroll_bar_state);
            }

            if let Some(scroll_area) = scroll_area_horizontal {
                let mut scroll_bar_state =
                    ScrollbarState::new(max_width).position(state.horizontal_offset as usize);

                Scrollbar::new(ratatui::widgets::ScrollbarOrientation::HorizontalBottom)
                    .symbols(ratatui::symbols::scrollbar::HORIZONTAL)
                    .render(scroll_area, buf, &mut scroll_bar_state);
            }
        }
    }
}

fn add_truncate(area: Rect, buf: &mut Buffer) {
    let truncated = Line::from("[truncated]")
        .right_aligned()
        .style(Style::default().dim().italic().bg(Color::DarkGray));
    let [_, last_line] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    let [_, truncated_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(truncated.width().try_into().unwrap_or(11)),
    ])
    .areas(last_line);
    truncated.render(truncated_area, buf);
}
