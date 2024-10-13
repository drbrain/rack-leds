use std::collections::VecDeque;

use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use text::ToLine;
use tokio::sync::{
    broadcast::{self, error::TryRecvError},
    mpsc::UnboundedSender,
    watch,
};

use crate::{
    tui_tracing::LogLine,
    ui::{widgets::Display, Action, Component, Config},
    Update,
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    updates: watch::Receiver<Vec<Update>>,
    tracing_receiver: broadcast::Receiver<LogLine>,
    log: VecDeque<LogLine>,
    log_max_lines: usize,
}

impl Home {
    pub fn new(
        updates: watch::Receiver<Vec<Update>>,
        tracing_receiver: broadcast::Receiver<LogLine>,
    ) -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            updates,
            tracing_receiver,
            log: Default::default(),
            log_max_lines: 50,
        }
    }

    fn trim_log(&mut self) {
        while self.log.len() > self.log_max_lines {
            self.log.pop_front();
        }
    }

    fn update_log(&mut self) {
        loop {
            match self.tracing_receiver.try_recv() {
                Ok(log_line) => self.log.push_back(log_line),
                Err(TryRecvError::Lagged(count)) => {
                    self.log.push_back(LogLine::missed(count));
                }
                Err(TryRecvError::Closed) | Err(TryRecvError::Empty) => break,
            }
            self.trim_log();
        }
    }

    fn update_log_max_lines(&mut self, height: usize) {
        self.log_max_lines = height.saturating_add(10);
        self.trim_log();
    }
}

impl Component for Home {
    fn init(&mut self, area: Size) -> Result<()> {
        self.update_log_max_lines(area.height.into());

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
                self.update_log();
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            Action::Resize(_, height) => {
                self.update_log_max_lines(height.into());
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

        draw_tracing(debug, frame, &self.log);

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

fn draw_tracing(debug_outer: Rect, frame: &mut Frame<'_>, log: &VecDeque<LogLine>) {
    let block = Block::new().title("Log").borders(Borders::ALL);
    let debug_inner = block.inner(debug_outer);

    let text: Vec<Line> = log.iter().map(|line| line.to_line()).collect();
    let text = Text::from(text);

    let text = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    // NOTE: Scrolling is hard https://github.com/ratatui/ratatui/issues/174
    let line_offset = text
        .line_count(debug_inner.width)
        .saturating_sub(debug_inner.height.into())
        .try_into()
        .unwrap_or(0);

    let text = text.scroll((line_offset, 0));

    frame.render_widget(text, debug_inner);
}
