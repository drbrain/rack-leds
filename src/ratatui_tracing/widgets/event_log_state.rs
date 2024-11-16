use std::{collections::VecDeque, time::Instant};

use tokio::sync::broadcast::error::TryRecvError;

use crate::ratatui_tracing::{
    widgets::FilterState, widgets::FormatState, Event, EventReceiver, Reloadable,
};

pub struct EventLogState<'a> {
    closed: bool,
    event_receiver: EventReceiver,
    pub(crate) filter: FilterState<'a>,
    pub(crate) format: FormatState,
    pub(crate) log: VecDeque<Event>,
    max_scrollback: usize,
}

impl<'a> EventLogState<'a> {
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

    pub(crate) fn epoch(&self) -> Instant {
        self.event_receiver.epoch
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
