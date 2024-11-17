use std::collections::{vec_deque, VecDeque};

use tokio::sync::broadcast::error::TryRecvError;

use crate::ratatui_tracing::Event;

use super::EventReceiver;

pub struct History {
    events: VecDeque<Event>,
    capacity: usize,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        let events = VecDeque::with_capacity(capacity);

        Self { events, capacity }
    }

    pub fn events(&self, count: usize) -> vec_deque::Iter<'_, Event> {
        let oldest = self.events.len().saturating_sub(count);

        self.events.range(oldest..)
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
                Ok(log_line) => self.events.push_back(log_line),
                Err(TryRecvError::Closed) => {
                    self.events.push_front(Event::closed());

                    return Err(try_recv.err().unwrap());
                }
                Err(TryRecvError::Lagged(count)) => {
                    self.events.push_front(Event::missed(count));
                }
                Err(TryRecvError::Empty) => break,
            }
        }

        Ok(())
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
