#![doc = include_str!("../README.md")]

#[macro_use]
mod util;
pub mod activity;
pub mod error;
mod handler;
mod io;
pub mod lobby;
pub mod overlay;
mod proto;
pub mod registration;
pub mod relations;
mod types;
pub mod user;
pub mod voice;

pub use error::{DiscordApiErr, DiscordErr, Error};
pub use handler::{handlers, wheel, DiscordHandler, DiscordMsg};
pub use proto::event::Event;
use proto::{Command, CommandKind};
pub use time::OffsetDateTime;
pub use types::Snowflake;
pub type AppId = i64;

pub use crossbeam_channel as cc;
use parking_lot::Mutex;
use std::sync::Arc;

/// The details on the [Application](https://discord.com/developers/docs/game-sdk/sdk-starter-guide#get-set-up)
/// you've created in Discord.
pub enum DiscordApp {
    /// Registers this application with Discord so that Discord can launch it
    /// to eg. join another user's game
    Register(registration::Application),
    /// The unique application id. Note that Discord will not be able launch
    /// this application when this variant is used, unless you've registered it
    /// in some other way
    PlainId(AppId),
}

impl From<AppId> for DiscordApp {
    fn from(id: AppId) -> Self {
        Self::PlainId(id)
    }
}

impl From<registration::Application> for DiscordApp {
    fn from(app: registration::Application) -> Self {
        Self::Register(app)
    }
}

bitflags::bitflags! {
    pub struct Subscriptions: u32 {
        const ACTIVITY = 0x1;
        const LOBBY = 0x2;
        const USER = 0x4;
        const OVERLAY = 0x8;
        const RELATIONSHIPS = 0x10;
        const VOICE = 0x20;

        const ALL = Self::ACTIVITY.bits | Self::LOBBY.bits | Self::USER.bits | Self::OVERLAY.bits | Self::RELATIONSHIPS.bits | Self::VOICE.bits;
    }
}

pub struct Discord {
    nonce: std::sync::atomic::AtomicUsize,
    /// Queue for messages to be sent to Discord
    send_queue: cc::Sender<Option<Vec<u8>>>,
    /// The handle to the task actually driving the I/O with Discord
    io_task: tokio::task::JoinHandle<()>,
    /// The handle to the task dispatching messages to the DiscordHandler
    handler_task: tokio::task::JoinHandle<()>,
    state: State,
}

impl Discord {
    /// Creates a new Discord connection for the specified application, providing
    /// a [`DiscordHandler`] which can handle events as they arrive from Discord
    pub fn new(
        app: impl Into<DiscordApp>,
        subscriptions: Subscriptions,
        handler: Box<dyn DiscordHandler>,
    ) -> Result<Self, Error> {
        let app_id = match app.into() {
            DiscordApp::PlainId(id) => id,
            DiscordApp::Register(inner) => {
                let id = inner.id;
                registration::register_app(inner)?;
                id
            }
        };

        let io_task = io::start_io_task(app_id);

        let state = State::default();

        let handler_task = handler::handler_task(
            handler,
            subscriptions,
            io_task.stx.clone(),
            io_task.rrx,
            state.clone(),
        );

        Ok(Self {
            nonce: std::sync::atomic::AtomicUsize::new(1),
            send_queue: io_task.stx,
            io_task: io_task.handle,
            handler_task,
            state,
        })
    }

    /// Disconnects from Discord, shutting down the tasks that have been created
    /// to handle sending and receiving messages from it.
    pub async fn disconnect(self) {
        let _ = self.send_queue.send(None);
        let _ = self.io_task.await;
        let _ = self.handler_task.await;
    }

    /// Serializes an RPC ands adds a notification oneshot so that we can be notified
    /// with the response from Discord
    fn send_rpc<Msg>(
        &self,
        cmd: CommandKind,
        msg: Msg,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<Command, Error>>, Error>
    where
        Msg: serde::Serialize,
    {
        // Increment the nonce, we use this in the handler task to pair the response
        // to this request
        let nonce = self
            .nonce
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let rpc = proto::Rpc {
            cmd,
            args: Some(msg),
            nonce: nonce.to_string(),
            evt: None,
        };

        let (tx, rx) = tokio::sync::oneshot::channel();

        self.state
            .notify_queue
            .lock()
            .push(NotifyItem { nonce, tx, cmd });

        let mut buffer = Vec::with_capacity(128);
        io::serialize_message(io::OpCode::Frame, &rpc, &mut buffer)?;
        self.send_queue.send(Some(buffer))?;

        Ok(rx)
    }
}

pub(crate) struct NotifyItem {
    /// The nonce we sent on the original request, the nonce in the response
    /// will be used to match this and remove it from the queue
    pub(crate) nonce: usize,
    /// The channel used to communicate back to the original caller of the RPC
    pub(crate) tx: tokio::sync::oneshot::Sender<Result<Command, Error>>,
    /// The expected command kind of the response, this is used to sanity check
    /// that Discord doesn't send us a response with a nonce that matches a
    /// different command
    pub(crate) cmd: CommandKind,
}

/// State shared between the top level [`Discord`] object and the handler task
#[derive(Clone)]
pub(crate) struct State {
    /// Queue of RPCs sent to Discord that are awaiting a response
    notify_queue: Arc<Mutex<Vec<NotifyItem>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            notify_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
