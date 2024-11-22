use std::time::Instant;

use eyre::Result;
use tokio::sync::broadcast::{
    self,
    error::{RecvError, TryRecvError},
};

use crate::ratatui_tracing::Event;

pub struct EventReceiver {
    pub epoch: Instant,
    pub channel: broadcast::Receiver<Event>,
}

impl EventReceiver {
    pub async fn recv(&mut self) -> Result<Event, RecvError> {
        self.channel.recv().await
    }

    pub fn resubscribe(&self) -> Self {
        EventReceiver {
            epoch: self.epoch,
            channel: self.channel.resubscribe(),
        }
    }

    pub fn try_recv(&mut self) -> Result<Event, TryRecvError> {
        self.channel.try_recv()
    }
}
