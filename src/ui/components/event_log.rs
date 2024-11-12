use eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Size},
    prelude::Rect,
    Frame,
};

use crate::{
    ratatui_tracing::EventReceiver,
    ui::{Action, Component},
};

pub struct EventLog {
    log: crate::ratatui_tracing::EventLog,
    show_format: bool,
}

impl EventLog {
    pub fn new(events: EventReceiver) -> Self {
        let log = crate::ratatui_tracing::EventLog::new(events, 50);

        Self {
            log,
            show_format: false,
        }
    }
}

impl Component for EventLog {
    fn init(&mut self, area: Size) -> Result<()> {
        self.log.set_max_lines(area.height.into());

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(&self.log, area);

        if self.show_format {
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

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
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
