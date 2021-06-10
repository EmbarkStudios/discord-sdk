//! See <https://github.com/discord/discord-rpc/blob/master/documentation/hard-mode.md>
//! for details on the protocol

// BEGIN - Embark standard lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.3
// BEGIN - Ark-specific lints
#![warn(
    clippy::ptr_as_ptr,
    clippy::path_buf_push_overwrite,
    clippy::explicit_deref_methods,
    clippy::expl_impl_clone_on_copy,
    clippy::zero_sized_map_values,
    clippy::invalid_upcast_comparisons,
    clippy::float_cmp_const,
    clippy::checked_conversions,
    clippy::char_lit_as_u8,
    clippy::fallible_impl_from,
    clippy::trait_duplication_in_bounds,
    clippy::wrong_pub_self_convention,
    clippy::same_functions_in_if_condition,
    clippy::mut_mut,
    clippy::string_lit_as_bytes,
    clippy::useless_transmute,
    clippy::manual_ok_or,
    clippy::mutex_integer
)]
// END - Ark-specific lints
// crate-specific exceptions:
#![allow()]

#[macro_use]
mod util;
mod activity;
pub mod error;
mod handler;
mod io;
mod lobby;
pub mod registration;
mod types;

pub use activity::{ActivityBuilder, Assets, IntoTimestamp, PartyPrivacy, Secrets};
pub use error::{DiscordApiErr, DiscordErr, Error};
pub use lobby::{Lobby, LobbyId};
use types::{Command, CommandKind};
pub use types::{Event, JoinReply, User};
pub type AppId = i64;

use crossbeam_channel as cc;
use parking_lot::{Mutex, RwLock};

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

bitflags::bitflags! {
    pub struct Subscriptions: u32 {
        const ACTIVITY = 0x1;
        const LOBBY = 0x2;
        const USER = 0x4;

        const ALL = Self::ACTIVITY.bits | Self::LOBBY.bits | Self::USER.bits;
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
    /// The application identifier. This is used for some RPCs sent to Discord.
    app_id: AppId,
    /// The lobbies owned by the current user
    owned_lobbies: RwLock<Vec<Lobby>>,
    /// The lobbies returned by the latest search
    searched_lobbies: RwLock<Vec<Lobby>>,
    state: State,
}

impl Discord {
    /// Creates a new Discord connection for the specified application, providing
    /// a [`DiscordHandler`] which can handle events as they arrive from Discord
    pub fn new(
        app: DiscordApp,
        subscriptions: Subscriptions,
        handler: Box<dyn DiscordHandler>,
    ) -> Result<Self, Error> {
        let app_id = match app {
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
            owned_lobbies: parking_lot::RwLock::new(Vec::new()),
            searched_lobbies: parking_lot::RwLock::new(Vec::new()),
            io_task: io_task.handle,
            handler_task,
            app_id,
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
    fn send_rpc<T: serde::Serialize>(
        &self,
        cmd: types::CommandKind,
        msg: T,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<Command, Error>>, Error> {
        // Increment the nonce, we use this in the handler to task to pair the response
        // to this request
        let nonce = self
            .nonce
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let rpc = handler::Rpc {
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

#[async_trait::async_trait]
pub trait DiscordHandler: Send + Sync {
    /// Method called when an event is received from Discord
    async fn on_event(&self, event: Event);
    /// Method called when an [`Error`] occurs when processing a response from
    /// Discord that can't be attributed to a specific request that was made
    async fn on_error(&self, error: Error);
}

use std::sync::Arc;

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
