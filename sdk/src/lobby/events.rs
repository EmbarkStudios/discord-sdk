use super::*;

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SpeakingEvent {
    /// The lobby with the voice channel
    pub lobby_id: LobbyId,
    /// The user in the lobby that started/stopped speaking
    pub user_id: UserId,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct MemberEvent {
    /// The lobby where the member state changed
    pub lobby_id: LobbyId,
    /// The details of the member that changed in the lobby
    pub member: LobbyMember,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct MessageEvent {
    /// The lobby the messsage was sent to
    pub lobby_id: LobbyId,
    /// The lobby member that sent the message
    pub sender_id: UserId,
    /// The message itself
    pub data: LobbyMessage,
}

#[derive(Debug, Clone)]
pub enum LobbyEvent {
    Create(Lobby),
    Connect(Lobby),
    /// Event fired when a user starts speaking in a lobby voice channel.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onspeaking)
    SpeakingStart(SpeakingEvent),
    /// Event fired when a user stops speaking in a lobby voice channel.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onspeaking)
    SpeakingStop(SpeakingEvent),
    /// Event fired when a user connects to a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberconnect)
    MemberConnect(MemberEvent),
    /// Event fired when a user disconnects from a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberdisconnect)
    MemberDisconnect(MemberEvent),
    /// Event fired when the metadata for a lobby member is changed.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onmemberupdate)
    MemberUpdate(MemberEvent),
    /// Event fired when a lobby is deleted, or when the current user disconnects.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbydelete)
    Delete {
        id: LobbyId,
    },
    /// Event fired when a lobby is updated. Note that this is only the metadata
    /// on the lobby itself, not the `members`.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbyupdate)
    Update(Lobby),
    /// Event fired when a message is sent to the lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#onlobbymessage)
    Message(MessageEvent),
}
