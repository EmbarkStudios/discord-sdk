#![doc = include_str!("../README.md")]
// BEGIN - Embark standard lints v5 for Rust 1.55+
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::rc_mutex,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.5 for Rust 1.55+
// crate-specific exceptions:

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
