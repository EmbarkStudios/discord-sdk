use crate::{
    activity::events as activity_events,
    lobby::{events as lobby_events, Lobby, LobbyId},
    overlay::events as overlay_events,
    types::ErrorPayload,
    user::events as user_events,
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
    /// Fires when we've done something naughty and Discord is telling us to stop.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord#error-handling)
    Error(ErrorPayload),

    /// Sent by Discord upon receipt of our [`Handshake`] message, the user is
    /// the current user logged in to the Discord we connected to.
    Ready(user_events::ConnectEvent),
    /// Fired when the connection has been interrupted between us and Discord,
    /// this is a synthesized event as there are can be numerous reasons on
    /// the client side for this to happen, in addition to Discord itself being
    /// closed, etc.
    #[serde(skip)]
    Disconnected { reason: crate::Error },
    /// Fired when any details on the current logged in user are changed.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/users#oncurrentuserupdate)
    CurrentUserUpdate(user_events::UpdateEvent),

    /// Event fired when a user starts speaking in a lobby voice channel.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onspeaking)
    SpeakingStart(lobby_events::SpeakingEvent),
    /// Event fired when a user stops speaking in a lobby voice channel.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onspeaking)
    SpeakingStop(lobby_events::SpeakingEvent),
    /// Event fired when a user connects to a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberconnect)
    LobbyMemberConnect(lobby_events::MemberEvent),
    /// Event fired when a user disconnects from a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberdisconnect)
    LobbyMemberDisconnect(lobby_events::MemberEvent),
    /// Event fired when a lobby is deleted, or the user disconnects from the lobby.
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
    LobbyMemberUpdate(lobby_events::MemberEvent),
    /// Event fired when a message is sent to the lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbymessage)
    LobbyMessage(lobby_events::MessageEvent),
    #[serde(skip)]
    LobbyCreate(Lobby),
    #[serde(skip)]
    LobbyConnect(Lobby),

    /// Sent by Discord when the local user has requested to join a game, and
    /// the remote user has accepted their request.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityjoin)
    ActivityJoin(activity_events::SecretEvent),
    /// Sent by Discord when the local user has chosen to spectate another user's
    /// game session.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityspectate)
    ActivitySpectate(activity_events::SecretEvent),
    /// Fires when a user asks to join the current user's game.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityjoinrequest)
    ActivityJoinRequest(activity_events::JoinRequestEvent),
    /// Fires when the current user is invited by another user to their game.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityinvite)
    ActivityInvite(activity_events::InviteEvent),

    /// Event fired when the overlay state changes.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/overlay#ontoggle)
    OverlayUpdate(overlay_events::UpdateEvent),
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

pub enum ClassifiedEvent {
    Lobby(lobby_events::LobbyEvent),
    User(user_events::UserEvent),
    Activity(activity_events::ActivityEvent),
    Overlay(overlay_events::OverlayEvent),
}

impl From<Event> for ClassifiedEvent {
    fn from(eve: Event) -> Self {
        use activity_events::ActivityEvent as AE;
        use lobby_events::LobbyEvent as LE;
        use user_events::UserEvent as UE;

        match eve {
            // User/connection
            Event::Ready(ce) => Self::User(UE::Connect(ce)),
            Event::Disconnected { reason } => {
                Self::User(UE::Disconnect(user_events::DisconnectEvent { reason }))
            }
            Event::CurrentUserUpdate(user) => Self::User(UE::Update(user)),

            // Lobby
            Event::SpeakingStart(se) => Self::Lobby(LE::SpeakingStart(se)),
            Event::SpeakingStop(se) => Self::Lobby(LE::SpeakingStop(se)),
            Event::LobbyDelete { id } => Self::Lobby(LE::Delete { id }),
            Event::LobbyUpdate(lob) => Self::Lobby(LE::Update(lob)),
            Event::LobbyMemberConnect(me) => Self::Lobby(LE::MemberConnect(me)),
            Event::LobbyMemberDisconnect(me) => Self::Lobby(LE::MemberDisconnect(me)),
            Event::LobbyMemberUpdate(me) => Self::Lobby(LE::MemberUpdate(me)),
            Event::LobbyMessage(msg) => Self::Lobby(LE::Message(msg)),
            Event::LobbyCreate(lobby) => Self::Lobby(LE::Create(lobby)),
            Event::LobbyConnect(lobby) => Self::Lobby(LE::Connect(lobby)),

            // Activity
            Event::ActivityJoin(secret) => Self::Activity(AE::Join(secret)),
            Event::ActivitySpectate(secret) => Self::Activity(AE::Spectate(secret)),
            Event::ActivityJoinRequest(jr) => Self::Activity(AE::JoinRequest(jr)),
            Event::ActivityInvite(inv) => Self::Activity(AE::Invite(inv)),

            // Overlay
            Event::OverlayUpdate(update) => {
                Self::Overlay(overlay_events::OverlayEvent::Update(update))
            }

            // Errors get converted before this path
            Event::Error(_) => unreachable!(),
        }
    }
}
