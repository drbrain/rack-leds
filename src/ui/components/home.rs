use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::{mpsc::UnboundedSender, watch};

use crate::{
    ratatui_tracing::{EventLog, EventReceiver},
    ui::{widgets::Display, Action, Component, Config},
    Update,
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    updates: watch::Receiver<Vec<Update>>,
    log: EventLog,
}

impl Home {
    pub fn new(updates: watch::Receiver<Vec<Update>>, event_receiver: EventReceiver) -> Self {
        let log = EventLog::new(event_receiver, 50);

        Self {
            command_tx: Default::default(),
            config: Default::default(),
            updates,
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
        let updates = self.updates.borrow().clone();

        let [status, display, debug] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(13),
            Constraint::Min(20),
        ])
        .areas(area);

        frame.render_widget(Paragraph::new("rack-leds"), status);

        let [display] = Layout::horizontal([Constraint::Length(55)]).areas(display);

        draw_display(display, frame, &updates);

        frame.render_widget(&self.log, debug);

        Ok(())
    }
}

fn draw_display(display_outer: Rect, frame: &mut Frame<'_>, updates: &[Update]) {
    let display = Block::new().title("Display").borders(Borders::ALL);
    let display_inner = display.inner(display_outer);
    frame.render_widget(display, display_outer);

    let heights: Vec<_> = updates.iter().map(|update| update.height()).collect();

    let layout = Layout::vertical(heights).split(display_inner);

    layout
        .iter()
        .zip(updates.iter())
        .for_each(|(area, update)| {
            let [area] = Layout::horizontal([update.width()]).split(*area)[..] else {
                unreachable!("Constraints removed from layout");
            };

            frame.render_widget(Display::new(update), area);
        });
}
