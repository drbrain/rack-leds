use std::{sync::Arc, time::Instant};

use crate::{
    widgets::{EventLogState, FormatState},
    Event,
};

pub struct DetailState {
    pub(crate) epoch: Instant,
    pub(crate) event: Arc<Event>,
    pub(crate) format: FormatState,
}

impl DetailState {
    pub(crate) fn new(event: Arc<Event>, epoch: Instant, format: FormatState) -> Self {
        Self {
            event,
            epoch,
            format,
        }
    }
}

impl<'a> From<&EventLogState<'a>> for DetailState {
    fn from(state: &EventLogState<'a>) -> Self {
        Self::new(state.selected_event(), state.epoch(), state.format.clone())
    }
}
