use std::{collections::VecDeque, time::Instant};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use tokio::sync::broadcast::error::TryRecvError;

use crate::ratatui_tracing::{
    widgets::FilterState, widgets::FormatState, Event, EventReceiver, Reloadable,
};

pub struct EventLog<'a> {
    closed: bool,
    event_receiver: EventReceiver,
    pub(crate) filter: FilterState<'a>,
    pub(crate) format: FormatState,
    log: VecDeque<Event>,
    max_scrollback: usize,
}

impl<'a> EventLog<'a> {
    pub fn new(
        event_receiver: EventReceiver,
        max_scrollback: usize,
        reloadable: Reloadable,
    ) -> Self {
        let filter = FilterState::new(reloadable);

        Self {
            closed: false,
            event_receiver,
            filter,
            format: Default::default(),
            log: Default::default(),
            max_scrollback,
        }
    }

    fn epoch(&self) -> Instant {
        self.event_receiver.epoch
    }

    pub fn log(&self) -> &VecDeque<Event> {
        &self.log
    }

    pub fn set_max_lines(&mut self, max_lines: usize) {
        self.max_scrollback = max_lines.saturating_add(10);

        self.trim();
    }

    pub fn trim(&mut self) {
        while self.log.len() > self.max_scrollback {
            self.log.pop_front();
        }
    }

    /// Move as many items as possible from the channel to the event log
    ///
    /// If events were missed when reading from the channel a missing event is synthesized
    pub fn update(&mut self) {
        if self.closed {
            return;
        }

        loop {
            match self.event_receiver.try_recv() {
                Ok(log_line) => self.log.push_back(log_line),
                Err(TryRecvError::Closed) => {
                    self.log.push_back(Event::closed());
                    self.trim();
                    self.closed = true;
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Lagged(count)) => {
                    self.log.push_back(Event::missed(count));
                }
            }

            self.trim();
        }
    }
}

impl<'a> Widget for &EventLog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let epoch = self.epoch();

        let block = Block::new().title("Log").borders(Borders::ALL);
        let block_inner = block.inner(area);

        let text: Vec<Line> = self
            .log
            .iter()
            .map(|line| line.to_line(epoch, &self.format))
            .collect();
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

        text.render(area, buf)
    }
}
