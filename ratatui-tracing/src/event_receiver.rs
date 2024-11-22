use std::time::Instant;

use eyre::Result;
use time::UtcOffset;
use tokio::sync::broadcast::{
    self,
    error::{RecvError, TryRecvError},
};

use crate::Event;

pub struct EventReceiver {
    pub channel: broadcast::Receiver<Event>,
    pub epoch: Instant,
    pub local_offset: Option<UtcOffset>,
}

impl EventReceiver {
    pub async fn recv(&mut self) -> Result<Event, RecvError> {
        self.channel.recv().await
    }

    pub fn resubscribe(&self) -> Self {
        let Self {
            channel,
            epoch,
            local_offset,
        } = self;

        EventReceiver {
            channel: channel.resubscribe(),
            epoch: *epoch,
            local_offset: *local_offset,
        }
    }

    pub fn try_recv(&mut self) -> Result<Event, TryRecvError> {
        self.channel.try_recv()
    }
}
