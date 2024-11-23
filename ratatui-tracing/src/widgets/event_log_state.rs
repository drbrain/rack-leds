use std::time::Instant;

use ratatui::layout::{Constraint, Layout, Rect};

use crate::{
    widgets::{FilterState, FormatState},
    EventReceiver, History, Reloadable,
};

/// State of an [`super::EventLog`] widget
///
/// This state contains:
/// * Whether active or historic events are visible
/// * Whether an event list or event detail is visible
/// * Event view formatting options ([`FilterState`])
/// * Editable filtering directives ([`FormatState`])
/// * Horizontal scroll position when wrapping is disabled
pub struct EventLogState<'a> {
    closed: bool,
    detail: bool,
    event_receiver: EventReceiver,
    /// Editable filtering directives
    pub filter: FilterState<'a>,
    /// List view format options
    pub format: FormatState,
    /// Horizontal scroll offset when wrapping is disabled
    pub horizontal_offset: u16,
    live_history: History,
    pause_history: Option<History>,
}

impl<'a> EventLogState<'a> {
    /// Create state for an [`EventReceiver`] and a [`Reloadable`].
    ///
    /// The `max_scrollback` controls how many events are stored in history.  If historic events
    /// are being viewed up to double this many events may be retained.
    pub fn new(
        event_receiver: EventReceiver,
        max_scrollback: usize,
        reloadable: Reloadable,
    ) -> Self {
        let filter = FilterState::new(reloadable);
        let format = (&event_receiver).into();
        let live_history = History::new(max_scrollback);

        Self {
            closed: false,
            detail: false,
            event_receiver,
            filter,
            format,
            horizontal_offset: 0,
            live_history,
            pause_history: None,
        }
    }

    /// Enable detail view for the selected event
    pub fn detail_show(&mut self) {
        self.detail = true;

        if self.is_live() {
            self.select_last();
        }
    }

    pub(crate) fn epoch(&self) -> Instant {
        self.event_receiver.epoch
    }

    /// Events viewable by this log
    pub fn history(&self) -> &History {
        self.pause_history.as_ref().unwrap_or(&self.live_history)
    }

    /// `true` when detail view is enabled
    pub fn is_detail(&self) -> bool {
        self.detail
    }

    /// `true` when live events are displayed
    pub fn is_live(&self) -> bool {
        self.pause_history.is_none()
    }

    /// Cancel detail view
    pub fn list_show(&mut self) {
        self.detail = false;
    }

    pub(crate) fn pause_history<F>(&mut self, f: F)
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

    pub(crate) fn scroll_area_horizontal(&self, area: Rect) -> (Rect, Option<Rect>) {
        if self.format.is_wrap() || self.horizontal_offset == 0 {
            (area, None)
        } else {
            let [area, scroll_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

            (area, Some(scroll_area))
        }
    }

    pub(crate) fn scroll_area_vertical(&self, area: Rect) -> (Rect, Option<Rect>) {
        if self.is_live() {
            (area, None)
        } else {
            let [area, scroll_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

            (area, Some(scroll_area))
        }
    }

    /// Scroll the column view left
    pub fn scroll_left(&mut self, columns: u16) {
        self.horizontal_offset = self.horizontal_offset.saturating_sub(columns);
    }

    /// Reset the scroll view to the left
    pub fn scroll_reset(&mut self) {
        self.horizontal_offset = 0;
    }

    /// Scroll the column view right
    pub fn scroll_right(&mut self, columns: u16) {
        self.horizontal_offset = self.horizontal_offset.saturating_add(columns);
    }

    /// Clear the selected item from history and return to live view
    pub fn select_clear(&mut self) {
        self.pause_history = None;
    }

    /// Pause history and select the first event
    ///
    /// Events will continue to accumulate in live history
    pub fn select_first(&mut self) {
        self.pause_history(|h| h.select_first());
    }

    /// Pause history and select the last event
    ///
    /// Events will continue to accumulate in live history
    pub fn select_last(&mut self) {
        self.pause_history(|h| h.select_last());
    }

    /// Pause history and select the next event toward the present
    ///
    /// Events will continue to accumulate in live history
    pub fn select_next(&mut self) {
        self.pause_history(|h| h.select_next());
    }

    /// Pause history and select the next event toward the past
    ///
    /// Events will continue to accumulate in live history
    pub fn select_previous(&mut self) {
        self.pause_history(|h| h.select_previous());
    }

    /// Set the max scrollback amount
    pub fn set_max_scrollback(&mut self, max_scrollback: usize) {
        self.live_history.set_capacity(max_scrollback);
    }

    /// The number of events stored in live history
    pub fn total(&self) -> usize {
        self.live_history.total()
    }

    /// Move as many items as possible from the channel to the event log
    ///
    /// Call this from your application event loop periodically to keep the event log updated
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

    pub fn wrap_toggle(&mut self) {
        self.format.wrap_toggle();
    }
}
