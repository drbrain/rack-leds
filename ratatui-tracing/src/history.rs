use std::{
    collections::{vec_deque, VecDeque},
    iter,
    sync::Arc,
};

use ratatui::widgets::ScrollbarState;
use tokio::sync::broadcast::error::TryRecvError;

use crate::{Event, EventReceiver};

pub type Iter<'a> = iter::Skip<iter::Enumerate<iter::Rev<vec_deque::Iter<'a, Arc<Event>>>>>;

#[derive(Clone)]
pub struct History {
    total: usize,
    events: VecDeque<Arc<Event>>,
    capacity: usize,
    pub(crate) selected: Option<usize>,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        let events = VecDeque::with_capacity(capacity);

        Self {
            total: 0,
            events,
            capacity,
            selected: None,
        }
    }

    pub fn events(&self) -> Iter<'_> {
        self.events.iter().rev().enumerate().skip(self.offset())
    }

    /// Insert events from the [EventReceiver] into history.
    ///
    /// If the [EventReceiver] is closed, returns a [TryRecvError]
    pub fn fill_from(&mut self, event_receiver: &mut EventReceiver) -> Result<(), TryRecvError> {
        loop {
            let try_recv = event_receiver.try_recv();

            while self.events.len() > self.capacity {
                self.events.pop_back();
            }

            match try_recv {
                Ok(log_line) => self.events.push_back(log_line.into()),
                Err(TryRecvError::Closed) => {
                    self.events.push_front(Event::closed().into());

                    return Err(try_recv.err().unwrap());
                }
                Err(TryRecvError::Lagged(count)) => {
                    self.events.push_front(Event::missed(count).into());
                }
                Err(TryRecvError::Empty) => break,
            }

            self.total += 1
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn offset(&self) -> usize {
        if let Some(selected) = self.selected {
            if selected >= self.events.len() {
                self.events.len().saturating_sub(1)
            } else {
                selected
            }
        } else {
            0
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn select_newer(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_sub(1));

        self.select(Some(next));
    }

    pub fn select_newest(&mut self) {
        self.select(Some(0));
    }

    pub fn select_older(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_add(1));

        self.select(Some(next));
    }

    pub fn select_oldest(&mut self) {
        self.select(Some(usize::MAX));
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        use std::cmp::Ordering::*;
        let current_capacity = self.capacity;

        match capacity.cmp(&current_capacity) {
            Less => {
                self.events.truncate(capacity);

                self.capacity = capacity;
            }
            Equal => (),
            Greater => {
                let additional = capacity.saturating_sub(current_capacity);

                self.events.reserve(additional);

                self.capacity = capacity;
            }
        }
    }

    pub fn total(&self) -> usize {
        self.total
    }
}

impl From<&History> for ScrollbarState {
    fn from(history: &History) -> Self {
        let content_length = history.len();
        let position = content_length.saturating_sub(history.offset());

        ScrollbarState::new(content_length).position(position)
    }
}
