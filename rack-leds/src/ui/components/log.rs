use eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Size},
    prelude::Rect,
    style::{Color, Stylize},
    text::Line,
    Frame,
};
use ratatui_tracing::{
    widgets::{EventLog, EventLogState, Filter, Format},
    EventReceiver, Reloadable,
};

use crate::ui::{widgets::Border, Action, Component};

#[derive(Default)]
enum ViewState {
    Filter,
    Format,
    #[default]
    None,
}

pub struct Log<'a> {
    pub(crate) log: EventLogState<'a>,
    view_state: ViewState,
}

impl<'a> Log<'a> {
    pub fn new(events: EventReceiver, reloadable: Reloadable) -> Self {
        let log = EventLogState::new(events, 1000, reloadable);

        Self {
            log,
            view_state: Default::default(),
        }
    }

    fn render_filter(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let [_, middle, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Min(60),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, center, _] =
            Layout::vertical([Constraint::Min(2), Constraint::Min(15), Constraint::Fill(1)])
                .areas(middle);

        let block = Border::new()
            .horizontal(1)
            .name("Filters")
            .help("Esc to dismiss")
            .build();

        frame.render_stateful_widget(Filter::default().block(block), center, &mut self.log.filter);
    }

    fn render_format(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let [_, middle, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Min(30),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, center, _] =
            Layout::vertical([Constraint::Min(2), Constraint::Min(15), Constraint::Fill(1)])
                .areas(middle);

        let block = Border::new()
            .horizontal(1)
            .name("Filters")
            .help("Esc to dismiss")
            .build();

        frame.render_stateful_widget(Format::default().block(block), center, &mut self.log.format);
    }
}

impl Component for Log<'_> {
    fn init(&mut self, area: Size) -> Result<()> {
        self.log.set_max_events(area.height.into());

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let status = if self.log.is_live() {
            Line::from("Live").bold()
        } else {
            Line::from("PAUSED").bold().fg(Color::Yellow)
        };

        let detail = if self.log.is_live() {
            let total = self.log.total();

            Line::from(format!("{total} events")).bold()
        } else {
            let total = self.log.total();
            let history = self.log.history();
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
            .bold()
            .fg(Color::Yellow)
        };

        let block = Border::new()
            .name("Log")
            .status(status)
            .detail(detail)
            .build();

        let event_log = EventLog::default().block(block);

        frame.render_stateful_widget(event_log, area, &mut self.log);

        match self.view_state {
            ViewState::Filter => {
                self.render_filter(area, frame);
            }
            ViewState::Format => {
                self.render_format(area, frame);
            }
            ViewState::None => (),
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::EventLogDetailShow => {
                self.log.detail_show();
            }
            Action::EventLogSelectClear => {
                self.log.select_clear();
            }
            Action::EventLogLast => {
                self.log.select_last();
            }
            Action::EventLogListShow => {
                self.log.list_show();
            }
            Action::EventLogNext => {
                self.log.select_next();
            }
            Action::EventLogPrevious => {
                self.log.select_previous();
            }
            Action::EventLogScrollLeft => {
                self.log.scroll_left();
            }
            Action::EventLogScrollLeftBig => {
                self.log.scroll_left_big();
            }
            Action::EventLogScrollReset => {
                self.log.scroll_reset();
            }
            Action::EventLogScrollRight => {
                self.log.scroll_right();
            }
            Action::EventLogScrollRightBig => {
                self.log.scroll_right_big();
            }
            Action::EventLogTop => {
                self.log.select_first();
            }
            Action::EventLogWrapToggle => {
                self.log.wrap_toggle();
            }
            Action::FilterAdd => {
                self.log.filter.add_start();
            }
            Action::FilterCancel => {
                self.log.filter.cancel();
            }
            Action::FilterDelete => {
                self.log.filter.delete_selected();
            }
            Action::FilterEdit => {
                self.log.filter.edit_start();
            }
            Action::FilterHide => {
                self.view_state = ViewState::None;
            }
            Action::FilterLast => {
                self.log.filter.row_last();
            }
            Action::FilterNext => {
                self.log.filter.row_next();
            }
            Action::FilterPrevious => {
                self.log.filter.row_previous();
            }
            Action::FilterShow => {
                self.view_state = ViewState::Filter;
            }
            Action::FilterSubmit => {
                self.log.filter.submit();
            }
            Action::FilterTop => {
                self.log.filter.row_first();
            }
            Action::FormatHide => {
                self.view_state = ViewState::None;
            }
            Action::FormatRowEdit => {
                self.log.format.row_edit();
            }
            Action::FormatRowLast => {
                self.log.format.row_last();
            }
            Action::FormatRowNext => {
                self.log.format.row_next();
            }
            Action::FormatRowTop => {
                self.log.format.row_first();
            }
            Action::FormatRowPrevious => {
                self.log.format.row_previous();
            }
            Action::FormatShow => {
                self.view_state = ViewState::Format;
            }
            Action::Input(key) => {
                self.log.filter.key(key);
            }
            Action::Resize(_, height) => {
                self.log.set_max_events(height.into());
            }
            Action::Tick => {
                self.log.update();
            }
            _ => {}
        }

        Ok(None)
    }
}
