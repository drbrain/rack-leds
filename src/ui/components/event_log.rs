use eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Size},
    prelude::Rect,
    Frame,
};

use crate::{
    ratatui_tracing::{
        widgets::{EventLogState, Filter, Format},
        EventReceiver, Reloadable,
    },
    ui::{Action, Component},
};

#[derive(Default)]
enum ViewState {
    Filter,
    Format,
    #[default]
    None,
}

pub struct EventLog<'a> {
    pub(crate) log: EventLogState<'a>,
    view_state: ViewState,
}

impl<'a> EventLog<'a> {
    pub fn new(events: EventReceiver, reloadable: Reloadable) -> Self {
        let log = EventLogState::new(events, 50, reloadable);

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

        frame.render_stateful_widget(Filter::default(), center, &mut self.log.filter);
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

        frame.render_stateful_widget(Format::default(), center, &mut self.log.format);
    }
}

impl Component for EventLog<'_> {
    fn init(&mut self, area: Size) -> Result<()> {
        self.log.set_max_lines(area.height.into());

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let event_log = crate::ratatui_tracing::widgets::EventLog::default();
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
                self.log.set_max_lines(height.into());
            }
            Action::Tick => {
                self.log.update();
            }
            _ => {}
        }

        Ok(None)
    }
}
