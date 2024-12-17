use serde::{Deserialize, Serialize};

/// The different RPC command types
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
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
    CloseActivityJoinRequest,
    /// RPC sent to invite another [`User`]
    ActivityInviteUser,
    /// RPC sent to accept the invite of another [`User`]
    AcceptActivityInvite,

    /// RPC sent to toggle the overlay either opened or closed
    #[serde(rename = "SET_OVERLAY_LOCKED")]
    SetOverlayVisibility,
    /// RPC sent to open the activity invite overlay modal
    OpenOverlayActivityInvite,
    /// RPC sent to open the guild invite overlay modal
    OpenOverlayGuildInvite,
    /// RPC sent to open the voice settings for the application
    OpenOverlayVoiceSettings,

    /// RPC sent to retrieve the full list of a user's active relationships
    GetRelationships,
}

/// The response to an RPC sent by us.
#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Command {
    Subscribe {
        evt: super::EventKind,
    },

    SetActivity(Box<Option<crate::activity::SetActivity>>),
    ActivityInviteUser,
    AcceptActivityInvite,
    SendActivityJoinInvite,
    CloseActivityJoinRequest,

    #[serde(rename = "SET_OVERLAY_LOCKED")]
    SetOverlayVisibility,
    OpenOverlayActivityInvite,
    OpenOverlayGuildInvite,
    OpenOverlayVoiceSettings,

    GetRelationships {
        relationships: Vec<crate::relations::Relationship>,
    },
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
