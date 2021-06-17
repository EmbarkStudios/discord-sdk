use crate::lobby::Lobby;
use serde::{Deserialize, Serialize};

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

/// The response to an RPC sent by us.
#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Command {
    Subscribe {
        evt: super::EventKind,
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
