use std::time::Instant;

use crate::ratatui_tracing::{
    history::History,
    widgets::{FilterState, FormatState},
    EventReceiver, Reloadable,
};

pub struct EventLogState<'a> {
    closed: bool,
    event_receiver: EventReceiver,
    pub(crate) filter: FilterState<'a>,
    pub(crate) format: FormatState,
    pub(crate) history: History,
}

impl<'a> EventLogState<'a> {
    pub fn new(
        event_receiver: EventReceiver,
        max_scrollback: usize,
        reloadable: Reloadable,
    ) -> Self {
        let filter = FilterState::new(reloadable);
        let history = History::new(max_scrollback);

        Self {
            closed: false,
            event_receiver,
            filter,
            format: Default::default(),
            history,
        }
    }

    pub(crate) fn epoch(&self) -> Instant {
        self.event_receiver.epoch
    }

    pub fn set_max_events(&mut self, max_events: usize) {
        self.history.set_capacity(max_events);
    }

    /// Move as many items as possible from the channel to the event log
    ///
    /// If events were missed when reading from the channel a missing event is synthesized
    pub fn update(&mut self) {
        if self.closed {
            return;
        }

        if self.history.fill_from(&mut self.event_receiver).is_err() {
            self.closed = true;
        }
    }
}
