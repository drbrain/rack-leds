use std::time::Instant;

use eyre::Result;
use time::UtcOffset;
use tokio::sync::broadcast::{
    self,
    error::{RecvError, TryRecvError},
};

use crate::Event;

/// Use `EventReceiver` to determine when to re-render the log view:
///
/// ```
/// # use ratatui_tracing::EventReceiver;
/// # use tracing::error;
/// # async {
/// let mut events: EventReceiver = todo!();
///
/// loop {
///     tokio::select! {
///         result = events.has_events() => {
///             if let Err(error) = result {
///                 error!(?error, "tracing event sender dropped");
///                 break;
///             }
///         }
///         // …
///     }
///
///     events = events.resubscribe();
///
///     // re-render EventLog widget
/// }
/// # };
/// ```
pub struct EventReceiver {
    pub(crate) channel: broadcast::Receiver<Event>,
    pub(crate) epoch: Instant,
    pub(crate) local_offset: Option<UtcOffset>,
}

impl EventReceiver {
    /// Returns `true` if there are new events, [`RecvError`] if the channel is closed.
    pub async fn has_events(&mut self) -> Result<bool, RecvError> {
        self.channel.recv().await.map(|_| true)
    }

    /// Re-subscribes to the channel, discarding unread items.
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

    pub(crate) fn try_recv(&mut self) -> Result<Event, TryRecvError> {
        self.channel.try_recv()
    }
}
