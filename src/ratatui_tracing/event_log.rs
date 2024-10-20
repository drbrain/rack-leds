use std::collections::VecDeque;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use text::ToLine;
use tokio::sync::broadcast::error::TryRecvError;

use crate::ratatui_tracing::{Event, EventReceiver};

pub struct EventLog {
    event_receiver: EventReceiver,
    log: VecDeque<Event>,
    max_lines: usize,
}

impl EventLog {
    pub fn new(event_receiver: EventReceiver, max_lines: usize) -> Self {
        Self {
            event_receiver,
            log: Default::default(),
            max_lines,
        }
    }

    pub fn log(&self) -> &VecDeque<Event> {
        &self.log
    }

    pub fn set_max_lines(&mut self, max_lines: usize) {
        self.max_lines = max_lines.saturating_add(10);

        self.trim();
    }

    pub fn trim(&mut self) {
        while self.log.len() > self.max_lines {
            self.log.pop_front();
        }
    }

    pub fn update(&mut self) {
        loop {
            match self.event_receiver.try_recv() {
                Ok(log_line) => self.log.push_back(log_line),
                Err(TryRecvError::Lagged(count)) => {
                    self.log.push_back(Event::missed(count));
                }
                Err(TryRecvError::Closed) | Err(TryRecvError::Empty) => break,
            }

            self.trim();
        }
    }
}

impl Widget for &EventLog {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new().title("Log").borders(Borders::ALL);
        let block_inner = block.inner(area);

        let text: Vec<Line> = self.log.iter().map(|line| line.to_line()).collect();
        let text = Text::from(text);

        let text = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

        // NOTE: Scrolling is hard https://github.com/ratatui/ratatui/issues/174
        // and Lists don't allow wrapping https://github.com/ratatui/ratatui/issues/128
        let line_offset = text
            .line_count(block_inner.width)
            .saturating_sub(block_inner.height.into())
            .try_into()
            .unwrap_or(0);

        let text = text.scroll((line_offset, 0));

        text.render(block_inner, buf)
    }
}
