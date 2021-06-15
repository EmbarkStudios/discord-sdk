use crate::{
    lobby::{self, Lobby, LobbyId},
    user::{User, UserId},
};
use serde::{Deserialize, Serialize};

pub type ChannelId = Snowflake;
pub type MessageId = Snowflake;

/// Message sent by Discord to close the connection
#[derive(Deserialize)]
pub(crate) struct CloseFrame<'frame> {
    pub(crate) code: Option<i32>,
    pub(crate) message: Option<&'frame str>,
}

#[derive(Deserialize, Debug)]
pub struct ErrorPayload {
    code: Option<u32>,
    message: Option<String>,
}

/// If we know the error type, we can convert it to a static error and avoid
/// doing a string copy
#[derive(Deserialize)]
pub(crate) struct ErrorPayloadStack<'stack> {
    pub(crate) code: Option<u32>,
    /// See https://github.com/serde-rs/serde/issues/1413#issuecomment-494892266
    /// for why this is a Cow, Discord occasionally sends escaped JSON in error
    /// messages
    pub(crate) message: Option<std::borrow::Cow<'stack, str>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Production,
    Other(String),
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    /// The CDN host that can be used to retrieve user avatars
    pub cdn_host: String,
    /// Supposedly this is the type of build of the Discord app, but seems
    /// to return "production" for stable, PTB, and canary, so not really
    /// useful
    pub environment: Environment,
    /// The url (well, not really because it doesn't specify the scheme
    /// for some reason) to the Discord REST API
    pub api_endpoint: String,
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

/// The response to an RPC sent by us.
#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Command {
    Subscribe {
        evt: EventKind,
    },

    CreateLobby(Lobby),
    UpdateLobby,
    SearchLobbies(Vec<Lobby>),
    DeleteLobby,
    ConnectToLobby(Lobby),
    DisconnectFromLobby,
    SendToLobby,
    ConnectToLobbyVoice,
    DisconnectFromLobbyVoice,
    UpdateLobbyMember,

    SetActivity(Box<Option<crate::activity::SetActivity>>),
    ActivityInviteUser,
    AcceptActivityInvite,

    #[serde(rename = "SET_OVERLAY_LOCKED")]
    SetOverlayVisibility,
    OpenOverlayActivityInvite,
    OpenOverlayGuildInvite,
    OpenOverlayVoiceSettings,
}

/// An RPC sent from Discord as JSON, in response to an RPC sent by us.
///
/// ```json
/// {
///     "cmd": "CREATE_LOBBY",
///     "evt": null,
///     "data": { "secret": "super_sekret" },
///     "nonce": "1",
/// }
/// ```
#[derive(Deserialize, Debug)]
pub(crate) struct CommandFrame {
    #[serde(flatten)]
    pub(crate) inner: Command,
    /// This nonce will match the nonce of the request from us that initiated
    /// this response
    #[serde(deserialize_with = "crate::util::string::deserialize")]
    pub(crate) nonce: usize,
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

/// The different RPC command types
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandKind {
    /// Dispatch the event specified in "evt".
    Dispatch,

    /// Subscribes to the event specified in "evt"
    Subscribe,
    /// Unsubscribes from the event specified in "evt"
    Unsubscribe,

    /// Updates the user's rich presence
    SetActivity,
    /// RPC sent when the local user has [`JoinReply::Accept`]ed a join request
    SendActivityJoinInvite,
    /// RPC sent when the local user has [`JoinReply::Reject`]ed a join request
    CloseActivityRequest,
    /// RPC sent to invite another [`User`]
    ActivityInviteUser,
    /// RPC sent to accept the invite of another [`User`]
    AcceptActivityInvite,

    /// RPC sent to create a lobby
    CreateLobby,
    /// RPC sent to modify the mutable properties of a lobby
    UpdateLobby,
    /// RPC sent to search for lobbies based on some criteria
    SearchLobbies,
    /// RPC sent to delete a lobby
    DeleteLobby,
    /// RPC sent to connect to a lobby
    ConnectToLobby,
    /// RPC sent to disconnect from a lobby
    DisconnectFromLobby,
    /// RPC to send a message to a lobby
    SendToLobby,
    /// RPC sent to join the current user to the voice channel of the specified lobby
    ConnectToLobbyVoice,
    /// RPC sent to disconnect the current user from the voice channel of the specified lobby
    DisconnectFromLobbyVoice,
    /// RPC sent to update a lobby member's metadata
    UpdateLobbyMember,

    /// RPC sent to toggle the overlay either opened or closed
    #[serde(rename = "SET_OVERLAY_LOCKED")]
    SetOverlayVisibility,
    /// RPC sent to open the activity invite overlay modal
    OpenOverlayActivityInvite,
    /// RPC sent to open the guild invite overlay modal
    OpenOverlayGuildInvite,
    /// RPC sent to open the voice settings for the application
    OpenOverlayVoiceSettings,
}

/// Discord uses [snowflakes](https://discord.com/developers/docs/reference#snowflakes)
/// for most/all of their unique identifiers, including users, lobbies, etc
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Snowflake(pub u64);

impl Snowflake {
    pub fn timestamp(self) -> chrono::DateTime<chrono::Utc> {
        let millis = self.0.overflowing_shr(22).0 + 1420070400000;
        let ts_seconds = millis / 1000;
        let ts_nanos = (millis % 1000) as u32 * 1000000;

        use chrono::TimeZone;
        chrono::Utc.timestamp(ts_seconds as i64, ts_nanos)
    }
}

use std::fmt;

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Snowflake {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Snowflake;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a u64 integer either as a number or a string")
            }

            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_u64(std::convert::TryInto::try_into(value).map_err(de::Error::custom)?)
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Snowflake(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Snowflake(value.parse().map_err(|e| {
                    de::Error::custom(format!("failed to parse u64: {}", e))
                })?))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
