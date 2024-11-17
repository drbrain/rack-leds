use std::{
    collections::{vec_deque, VecDeque},
    iter,
    sync::Arc,
};

use tokio::sync::broadcast::error::TryRecvError;

use crate::ratatui_tracing::{Event, EventReceiver};

pub type Iter<'a> = iter::Skip<iter::Enumerate<iter::Rev<vec_deque::Iter<'a, Arc<Event>>>>>;

#[derive(Clone)]
pub struct History {
    events: VecDeque<Arc<Event>>,
    capacity: usize,
    pub(crate) selected: Option<usize>,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        let events = VecDeque::with_capacity(capacity);

        Self {
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

    pub fn select_first(&mut self) {
        self.select(Some(usize::MAX));
    }

    pub fn select_last(&mut self) {
        self.select(Some(0));
    }

    pub fn select_next(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_sub(1));

        self.select(Some(next));
    }

    pub fn select_previous(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_add(1));

        self.select(Some(next));
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
}
