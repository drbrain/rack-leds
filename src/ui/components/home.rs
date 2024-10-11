use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::{mpsc::UnboundedSender, watch};

use crate::{
    ui::{widgets::Display, Action, Component, Config},
    Update,
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    updates: watch::Receiver<Vec<Update>>,
}

impl Home {
    pub fn new(updates: watch::Receiver<Vec<Update>>) -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            updates,
        }
    }
}

impl Component for Home {
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
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
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

        let text = draw_display(display, frame, &updates);

        draw_debug(debug, frame, text);

        Ok(())
    }
}

fn draw_debug(debug_outer: Rect, frame: &mut Frame<'_>, text: Vec<String>) {
    let debug = Block::new().title("Debug").borders(Borders::ALL);
    let debug_inner = debug.inner(debug_outer);
    frame.render_widget(debug, debug_outer);

    let text: Vec<Line> = text.iter().map(|l| Line::from(l.as_str())).collect();
    let text = Text::from(text);

    frame.render_widget(text, debug_inner);
}

fn draw_display(display_outer: Rect, frame: &mut Frame<'_>, updates: &[Update]) -> Vec<String> {
    let display = Block::new().title("Display").borders(Borders::ALL);
    let display_inner = display.inner(display_outer);
    frame.render_widget(display, display_outer);

    let heights: Vec<_> = updates.iter().map(|update| update.height()).collect();

    let layout = Layout::vertical(heights).split(display_inner);

    let mut lines = vec![];

    //updates.iter().map(|u| format!("{u:?}").into()).collect();

    layout
        .iter()
        .zip(updates.iter())
        .for_each(|(area, update)| {
            let [layout] = Layout::horizontal([update.width()]).split(*area)[..] else {
                unreachable!("Constraints removed from layout");
            };

            lines.push(format!(
                "layout: {:?}, x: {} y: {}",
                layout,
                update.x_bound(),
                update.y_bound(),
            ));

            frame.render_widget(Display::new(update), layout);
        });

    lines
}
