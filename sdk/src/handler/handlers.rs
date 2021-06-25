use super::{DiscordHandler, DiscordMsg};
use async_trait::async_trait;

/// Prints events at [`tracing::Level::DEBUG`] and errors at [`tracing::Level::WARN`]
pub struct Printer;

#[async_trait]
impl DiscordHandler for Printer {
    async fn on_message(&self, msg: DiscordMsg) {
        match msg {
            DiscordMsg::Event(eve) => tracing::debug!(event = ?eve),
            DiscordMsg::Error(err) => tracing::warn!(error = ?err),
        }
    }
}

/// Forwards messages to a receiver
pub struct Forwarder {
    tx: tokio::sync::mpsc::UnboundedSender<DiscordMsg>,
}

impl Forwarder {
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<DiscordMsg>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        (Self { tx }, rx)
    }
}

#[async_trait]
impl DiscordHandler for Forwarder {
    async fn on_message(&self, msg: DiscordMsg) {
        if let Err(msg) = self.tx.send(msg) {
            tracing::warn!(msg = ?msg.0, "message dropped");
        }
    }
}
