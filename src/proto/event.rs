use crate::{
    lobby::{self, Lobby, LobbyId},
    types::{DiscordConfig, ErrorPayload},
    user::{User, UserId},
};
use serde::{Deserialize, Serialize};

/// Events sent from Discord when some action occurs
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum EventKind {
    Ready,
    Error,

    CurrentUserUpdate,

    ActivityJoinRequest,
    ActivityJoin,
    ActivitySpectate,
    ActivityInvite,

    LobbyUpdate,
    LobbyDelete,
    LobbyMemberConnect,
    LobbyMemberUpdate,
    LobbyMemberDisconnect,
    LobbyMessage,
    SpeakingStart,
    SpeakingStop,

    OverlayUpdate,
}

/// An event sent from Discord to notify us of some kind of state change or
/// completed action.
///
/// ```json
/// { "evt": "ACTIVITY_JOIN", "data": { "secret": "super_sekret" } }
/// ```
#[derive(Deserialize, Debug)]
#[serde(tag = "evt", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    /// Sent by Discord upon receipt of our [`Handshake`] message, the user is
    /// the current user logged in to the Discord we connected to.
    Ready {
        /// The protocol version, we only support v1, which is fine since that is
        /// (currently) the only version
        #[serde(rename = "v")]
        version: u32,
        config: DiscordConfig,
        /// The user that is logged into the Discord application we connected to
        #[serde(deserialize_with = "crate::user::de_user")]
        user: User,
    },
    /// Fires when we've done something naughty and Discord is telling us to stop.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord#error-handling)
    Error(ErrorPayload),
    /// Fired when the connection has been interrupted between us and Discord
    #[serde(skip)]
    Disconnected { reason: String },

    /// Fired when any details on the current logged in user are changed.
    CurrentUserUpdate {
        #[serde(flatten, deserialize_with = "crate::user::de_user")]
        user: User,
    },

    /// Sent by Discord when the local user has requested to join a game, and
    /// the remote user has accepted their request.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityjoin)
    ActivityJoin { secret: String },
    /// Sent by Discord when the local user has chosen to spectate another user's
    /// game session.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityspectate)
    ActivitySpectate { secret: String },
    /// Fires when a user asks to join the current user's game.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityjoinrequest)
    ActivityJoinRequest {
        #[serde(deserialize_with = "crate::user::de_user")]
        user: User,
    },
    /// Fires when the current user is invited by another user to their game.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityinvite)
    ActivityInvite(Box<crate::activity::ActivityInvite>),

    /// Event fired when a user starts speaking in a lobby voice channel.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onspeaking)
    SpeakingStart {
        /// The lobby with the voice channel
        lobby_id: LobbyId,
        /// The user in the lobby that started speaking
        user_id: UserId,
    },
    /// Event fired when a user stops speaking in a lobby voice channel.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onspeaking)
    SpeakingStop {
        /// The lobby with the voice channel
        lobby_id: LobbyId,
        /// The user in the lobby that started speaking
        user_id: UserId,
    },
    /// Event fired when a user connects to a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberconnect)
    LobbyMemberConnect {
        /// The lobby the user connected to
        lobby_id: LobbyId,
        /// The details of the member that connected to the lobby
        member: lobby::LobbyMember,
    },
    /// Event fired when a user disconnects from a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberdisconnect)
    LobbyMemberDisconnect {
        /// The lobby the user disconnected from
        lobby_id: LobbyId,
        /// The details of the member that disconnected from the lobby
        member: lobby::LobbyMember,
    },
    /// Event fired when a lobby is deleted.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbydelete)
    LobbyDelete { id: LobbyId },
    /// Event fired when a lobby is updated. Note that this is only the metadata
    /// on the lobby itself, not the `members`.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbyupdate)
    LobbyUpdate(Lobby),
    /// Event fired when the metadata for a lobby member is changed.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberupdate)
    LobbyMemberUpdate {
        /// The lobby that contains the member that was updated
        lobby_id: LobbyId,
        /// The updated member
        member: lobby::LobbyMember,
    },
    /// Event fired when a message is sent to the lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbymessage)
    LobbyMessage {
        /// The lobby the messsage was sent to
        lobby_id: LobbyId,
        /// The lobby member that sent the message
        sender_id: UserId,
        /// The message itself
        data: lobby::LobbyMessage,
    },

    /// Event fired when the overlay state changes.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/overlay#ontoggle)
    OverlayUpdate {
        /// Whether the user has the overlay enabled or disabled. If the overlay
        /// is disabled, all the functionality of the SDK will still work. The
        /// calls will instead focus the Discord client and show the modal there
        /// instead of in application.
        enabled: bool,
        /// Whether the overlay is visible or not.
        #[serde(rename = "locked")]
        visible: crate::overlay::Visibility,
    },
}

/// An event sent from Discord as JSON.
///
/// ```json
/// {
///     "cmd": "DISPATCH",
///     "evt": "ACTIVITY_JOIN",
///     "data": { "secret": "super_sekret" },
///     "nonce": null,
/// }
/// ```
#[derive(Deserialize, Debug)]
pub(crate) struct EventFrame {
    /// The actual data payload, we don't care about "cmd" or "nonce" since
    /// nonce is not set for events and cmd is always `DISPATCH`.
    #[serde(flatten)]
    pub(crate) inner: Event,
}
