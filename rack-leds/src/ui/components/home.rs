use std::collections::HashMap;

use bytes::Bytes;
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use ratatui_tracing::{EventReceiver, Reloadable};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::{
    collector::UpdateReceiver,
    device::Id,
    png_builder::PngSender,
    ui::{components::Log, widgets::Display, Action, Component, Config},
    Columns, PngBuilder, Update,
};

pub struct Home<'a> {
    columns: Columns,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    updates: UpdateReceiver,
    png_sender: PngSender,
    log: Log<'a>,
}

impl<'a> Home<'a> {
    pub fn new(
        columns: Columns,
        updates: UpdateReceiver,
        png_sender: PngSender,
        events: EventReceiver,
        reloadable: Reloadable,
    ) -> Self {
        let log = Log::new(events, reloadable);

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

impl Component for Home<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(Clear, area);
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

        self.log.draw(frame, debug)?;

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        self.log.update(action)
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
