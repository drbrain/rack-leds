use std::collections::HashMap;

use bytes::Bytes;
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::{
    collector::UpdateReceiver,
    device::Id,
    png_builder::PngSender,
    ratatui_tracing::{EventLog, EventReceiver},
    ui::{widgets::Display, Action, Component, Config},
    Columns, PngBuilder, Update,
};

pub struct Home {
    columns: Columns,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    updates: UpdateReceiver,
    png_sender: PngSender,
    log: EventLog,
}

impl Home {
    pub fn new(
        columns: Columns,
        updates: UpdateReceiver,
        png_sender: PngSender,
        event_receiver: EventReceiver,
    ) -> Self {
        let log = EventLog::new(event_receiver, 50);

        Self {
            columns,
            command_tx: Default::default(),
            config: Default::default(),
            updates,
            png_sender,
            log,
        }
    }
}

impl Component for Home {
    fn init(&mut self, area: Size) -> Result<()> {
        self.log.set_max_lines(area.height.into());

        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.log.update();
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            Action::Resize(_, height) => {
                self.log.set_max_lines(height.into());
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let update_png = self.updates.has_changed().unwrap_or(false);
        let (updates, updated_at) = self.updates.borrow().clone();

        let [status, display, debug] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(13),
            Constraint::Min(20),
        ])
        .areas(area);

        frame.render_widget(Paragraph::new("rack-leds"), status);

        let [display] = Layout::horizontal([Constraint::Length(55)]).areas(display);

        if let Some(png) = draw_display(display, frame, &self.columns, updates, update_png) {
            self.png_sender.send_replace((png, updated_at));
        };

        frame.render_widget(&self.log, debug);

        Ok(())
    }
}

fn draw_display(
    display_outer: Rect,
    frame: &mut Frame<'_>,
    columns: &Columns,
    updates: HashMap<Id, Update>,
    update_png: bool,
) -> Option<Bytes> {
    let display = Block::new().title("Display").borders(Borders::ALL);
    let display_inner = display.inner(display_outer);
    frame.render_widget(display, display_outer);

    let widths: Vec<_> = columns
        .columns()
        .map(|column| {
            column
                .ids()
                .filter_map(|id| updates.get(&id))
                .map(|update| update.width())
                .max()
                .unwrap_or(0)
        })
        .collect();

    let column_rects = Layout::horizontal(Constraint::from_lengths(widths)).split(display_inner);

    column_rects
        .iter()
        .zip(columns.columns())
        .for_each(|(area, column)| {
            let updates: Vec<_> = column.ids().filter_map(|id| updates.get(&id)).collect();

            let heights: Vec<_> = updates.iter().map(|update| update.height()).collect();

            let layout = Layout::vertical(heights).split(*area);

            layout
                .iter()
                .zip(updates.iter())
                .for_each(|(area, update)| {
                    let [area] = Layout::horizontal([update.width()]).split(*area)[..] else {
                        unreachable!("Constraints removed from layout");
                    };

                    frame.render_widget(Display::new(update), area);
                });
        });

    if update_png {
        match PngBuilder::new(frame, &display_inner).build() {
            Ok(png) => Some(png),
            Err(e) => {
                error!(?e, "error building PNG");
                None
            }
        }
    } else {
        None
    }
}
