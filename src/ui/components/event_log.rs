use eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Size},
    prelude::Rect,
    Frame,
};

use crate::{
    ratatui_tracing::{EventReceiver, Filter, Reloadable},
    ui::{Action, Component},
};

pub struct EventLog<'a> {
    log: crate::ratatui_tracing::EventLog,
    filter: Filter<'a>,
    show_filter: bool,
    show_format: bool,
}

impl<'a> EventLog<'a> {
    pub fn new(events: EventReceiver, reloadable: Reloadable) -> Self {
        let log = crate::ratatui_tracing::EventLog::new(events, 50);
        let filter = Filter::new(reloadable);

        Self {
            log,
            filter,
            show_filter: false,
            show_format: false,
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

        frame.render_widget(&self.filter, center);
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

        frame.render_widget(&self.log.format(), center);
    }
}

impl Component for EventLog<'_> {
    fn init(&mut self, area: Size) -> Result<()> {
        self.log.set_max_lines(area.height.into());

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(&self.log, area);

        if self.show_format {
            self.render_format(area, frame);
        }

        if self.show_filter {
            self.render_filter(area, frame);
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FilterAdd => {
                self.filter.add_start();
            }
            Action::FilterCancel => {
                self.filter.cancel();
            }
            Action::FilterDelete => {
                self.filter.delete_selected();
            }
            Action::FilterEdit => {
                self.filter.edit_start();
            }
            Action::FilterHide => {
                self.show_filter = false;
            }
            Action::FilterLast => {
                self.filter.row_last();
            }
            Action::FilterNext => {
                self.filter.row_next();
            }
            Action::FilterPrevious => {
                self.filter.row_previous();
            }
            Action::FilterShow => {
                self.show_filter = true;
            }
            Action::FilterSubmit => {
                self.filter.submit();
            }
            Action::FilterTop => {
                self.filter.row_first();
            }
            Action::FormatHide => {
                self.show_format = false;
            }
            Action::FormatRowEdit => {
                self.log.format().row_edit();
            }
            Action::FormatRowLast => {
                self.log.format().row_last();
            }
            Action::FormatRowNext => {
                self.log.format().row_next();
            }
            Action::FormatRowTop => {
                self.log.format().row_first();
            }
            Action::FormatRowPrevious => {
                self.log.format().row_previous();
            }
            Action::FormatShow => {
                self.show_format = true;
            }
            Action::Input(key) => {
                self.filter.key(key);
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
