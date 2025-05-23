use std::convert::Into;

use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Wrap},
};

use crate::widgets::{CreateFilter, Detail, EventLogState};

/// Widget to display events forwarded to an [`crate::EventReceiver`]
///
/// [`EventLog`] is a stateful widget that uses [`EventLogState`] for displaying recent or past
/// events and selecting an event for a detail view.
pub struct EventLog<'a> {
    block: Option<Block<'a>>,
    highlight_style: Style,
}

impl<'a> EventLog<'a> {
    /// Wrap the log with a [`Block`] widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    /// Set the highlight style for the selected log event
    pub fn highlight_style(mut self, highlight_style: impl Into<Style>) -> Self {
        self.highlight_style = highlight_style.into();

        self
    }
}

impl<'a> Default for EventLog<'a> {
    /// The default `EventLog` uses a [`Color::DarkGray`] [`Self::highlight_style()`]
    fn default() -> Self {
        let highlight_style = Style::default().bg(Color::DarkGray);

        Self {
            block: None,
            highlight_style,
        }
    }
}

impl<'a> StatefulWidget for EventLog<'a> {
    type State = EventLogState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = if let Some(block) = self.block {
            let log_area = block.inner(area);
            block.render(area, buf);

            log_area
        } else {
            area
        };

        let (area, scroll_area_vertical) = state.scroll_area_vertical(area);

        if let Some(detail_state) = state.detail_state() {
            Detail::default().render(area, buf, detail_state);
        } else if let Some(create_filter_state) = state.create_filter_state() {
            CreateFilter::default().render(area, buf, create_filter_state);
        } else {
            let selected = state.history().selected;
            let epoch = state.epoch();

            let events = state
                .history()
                .events()
                .map(|(i, event)| (i, event.to_line(epoch, &state.format)));

            let (area, scroll_area_horizontal) = state.scroll_area_horizontal(area);

            let mut current_height = 0;
            let mut max_width = 0;
            let mut visible = 0;

            for (i, event) in events {
                visible += 1;
                max_width = max_width.max(event.width());

                let event = Paragraph::new(event);

                let event = if state.format.is_wrap() {
                    event.wrap(Wrap { trim: false })
                } else {
                    event.scroll((0, state.horizontal_offset))
                };

                let mut truncate = false;
                let remaining_height = area.height.saturating_sub(current_height);
                let height = event.line_count(area.width) as u16;

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
                    x: area.left(),
                    y: area.bottom() - current_height,
                    width: area.width,
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
