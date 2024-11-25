use std::{sync::Arc, time::Instant};

use ratatui::layout::{Constraint, Layout, Rect};

use crate::{
    widgets::{CreateFilterState, DetailState, FilterState, FormatState},
    Event, EventReceiver, History, Reloadable,
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
    event_receiver: EventReceiver,
    /// Editable filtering directives
    pub filter: FilterState<'a>,
    /// List view format options
    pub format: FormatState,
    /// Horizontal scroll offset when wrapping is disabled
    pub horizontal_offset: u16,
    live_history: History,
    pause_history: Option<History>,
    view: ViewState,
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
            event_receiver,
            filter,
            format,
            horizontal_offset: 0,
            live_history,
            pause_history: None,
            view: Default::default(),
        }
    }

    pub fn create_filter(&mut self) {
        if self.is_live() {
            self.select_newest();
        }

        self.view = self.view.to_create_filter(self);
    }

    pub fn create_filter_state(&mut self) -> Option<&mut CreateFilterState> {
        self.view.create_filter_state()
    }

    pub fn detail_state(&mut self) -> Option<&mut DetailState> {
        self.view.detail_state()
    }

    /// Enable detail view for the selected event
    pub fn detail_show(&mut self) {
        if self.is_live() {
            self.select_newest();
        }

        self.view = self.view.to_detail(self);
    }

    pub(crate) fn epoch(&self) -> Instant {
        self.event_receiver.epoch
    }

    pub fn filter_create_next(&mut self) {
        if let Some(create_filter) = self.create_filter_state() {
            create_filter.select_next();
        }
    }

    pub fn filter_create_toggle(&mut self) {
        if let Some(create_filter) = self.create_filter_state() {
            create_filter.toggle();
        }
    }

    /// Events viewable by this log
    pub fn history(&self) -> &History {
        self.pause_history.as_ref().unwrap_or(&self.live_history)
    }

    /// `true` after the event sender is closed
    ///
    /// [`Self::is_closed()`] must be called regularly to detect a closed sender
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// `true` when detail view is enabled
    pub fn is_detail(&self) -> bool {
        self.view.is_detail()
    }

    /// `true` when live events are displayed
    pub fn is_live(&self) -> bool {
        self.pause_history.is_none()
    }

    /// Cancel detail view
    pub fn list_show(&mut self) {
        self.view = self.view.to_list();
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

    /// Pause history and select a newer event than the selected event
    ///
    /// Events will continue to accumulate in live history
    pub fn select_newer(&mut self) {
        self.pause_history(|h| h.select_newer());
    }

    /// Pause history and select the newest event
    ///
    /// Events will continue to accumulate in live history
    pub fn select_newest(&mut self) {
        self.pause_history(|h| h.select_newest());
    }

    /// Pause history and select an older event than the selected event
    ///
    /// Events will continue to accumulate in live history
    pub fn select_older(&mut self) {
        self.pause_history(|h| h.select_older());
    }

    /// Pause history and select the oldest event
    ///
    /// Events will continue to accumulate in live history
    pub fn select_oldest(&mut self) {
        self.pause_history(|h| h.select_oldest());
    }

    pub(crate) fn selected_event(&self) -> Arc<Event> {
        let selected = self.history().selected.unwrap_or(0);

        self.history()
            .events()
            .nth(selected)
            .map(|(_, event)| event)
            .cloned()
            .unwrap_or_else(|| Event::dropped(selected, self.total()).into())
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

    /// Toggle log line wrapping
    pub fn wrap_toggle(&mut self) {
        self.format.wrap_toggle();
    }
}

#[derive(Default)]
enum ViewState {
    CreateFilter {
        state: CreateFilterState,
    },
    Detail {
        state: DetailState,
    },
    #[default]
    List,
}

impl ViewState {
    fn create_filter_state(&mut self) -> Option<&mut CreateFilterState> {
        match self {
            ViewState::CreateFilter { state } => Some(state),
            _ => None,
        }
    }

    fn detail_state(&mut self) -> Option<&mut DetailState> {
        match self {
            ViewState::Detail { state } => Some(state),
            _ => None,
        }
    }

    fn is_detail(&self) -> bool {
        matches!(self, Self::Detail { .. })
    }

    fn to_create_filter(&self, state: &EventLogState) -> Self {
        Self::CreateFilter {
            state: state.into(),
        }
    }

    fn to_detail(&self, state: &EventLogState) -> Self {
        Self::Detail {
            state: state.into(),
        }
    }

    fn to_list(&self) -> Self {
        Self::List
    }
}
