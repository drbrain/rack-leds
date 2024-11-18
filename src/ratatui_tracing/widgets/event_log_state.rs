use std::time::Instant;

use crate::ratatui_tracing::{
    widgets::{FilterState, FormatState},
    EventReceiver, History, Reloadable,
};

pub struct EventLogState<'a> {
    closed: bool,
    detail: bool,
    event_receiver: EventReceiver,
    pub(crate) filter: FilterState<'a>,
    pub(crate) format: FormatState,
    live_history: History,
    pause_history: Option<History>,
}

impl<'a> EventLogState<'a> {
    pub fn new(
        event_receiver: EventReceiver,
        max_scrollback: usize,
        reloadable: Reloadable,
    ) -> Self {
        let filter = FilterState::new(reloadable);
        let live_history = History::new(max_scrollback);

        Self {
            closed: false,
            detail: false,
            event_receiver,
            filter,
            format: Default::default(),
            live_history,
            pause_history: None,
        }
    }

    pub fn detail_show(&mut self) {
        self.detail = true;

        if self.is_live() {
            self.select_last();
        }
    }

    pub(crate) fn epoch(&self) -> Instant {
        self.event_receiver.epoch
    }

    pub(crate) fn history(&self) -> &History {
        self.pause_history.as_ref().unwrap_or(&self.live_history)
    }

    pub fn is_detail(&self) -> bool {
        self.detail
    }

    pub fn is_live(&self) -> bool {
        self.pause_history.is_none()
    }

    pub fn list_show(&mut self) {
        self.detail = false;
    }

    pub fn pause_history<F>(&mut self, f: F)
    where
        F: FnOnce(&mut History),
    {
        if self.pause_history.is_none() {
            self.pause_history = Some(self.live_history.clone());
        }

        if let Some(ref mut history) = &mut self.pause_history {
            f(history)
        }
    }

    pub fn select_clear(&mut self) {
        self.pause_history = None;
    }

    pub fn select_first(&mut self) {
        self.pause_history(|h| h.select_first());
    }

    pub fn select_last(&mut self) {
        self.pause_history(|h| h.select_last());
    }

    pub fn select_next(&mut self) {
        self.pause_history(|h| h.select_next());
    }

    pub fn select_previous(&mut self) {
        self.pause_history(|h| h.select_previous());
    }

    pub fn set_max_events(&mut self, max_events: usize) {
        self.live_history.set_capacity(max_events);
    }

    pub fn total(&self) -> usize {
        self.live_history.total()
    }

    /// Move as many items as possible from the channel to the event log
    ///
    /// If events were missed when reading from the channel a missing event is synthesized
    pub fn update(&mut self) {
        if self.closed {
            return;
        }

        if self
            .live_history
            .fill_from(&mut self.event_receiver)
            .is_err()
        {
            self.closed = true;
        }
    }
}