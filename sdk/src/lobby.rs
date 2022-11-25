//! Provides types and functionality for [Lobbies](https://discord.com/developers/docs/game-sdk/lobbies)

pub mod events;
pub mod search;
pub mod state;

use crate::{types::Snowflake, user::UserId, Command, CommandKind, Error};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Metadata = std::collections::BTreeMap<String, String>;
pub type LobbyId = Snowflake;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    Amsterdam,
    Brazil,
    Dubai,
    EuCentral,
    EuWest,
    Europe,
    Frankfurt,
    Hongkong,
    India,
    Japan,
    London,
    Russia,
    Singapore,
    Southafrica,
    SouthKorea,
    Stockholm,
    Sydney,
    UsCentral,
    UsEast,
    UsSouth,
    UsWest,
    VipAmsterdam,
    VipUsEast,
    VipUsWest,
    // This isn't in the list returned by /voice/regions but...
    StPete,
}

#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LobbyKind {
    Private = 1,
    Public = 2,
}

/// The voice states that can be attached to each lobby member
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct VoiceState {
    pub channel_id: crate::types::ChannelId,
    pub deaf: bool,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    pub self_video: bool,
    pub session_id: String,
    pub suppress: bool,
    pub user_id: UserId,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Lobby {
    /// The unique identifier for the lobby.
    pub id: LobbyId,
    /// The maximum number of users that can join the lobby.
    pub capacity: u32,
    /// Whether new members can join the lobby.
    pub locked: bool,
    /// The users and attached metadata that are actually present in the lobby.
    /// This list will be empty if this lobby is deserialized from a
    /// [`LobbyUpdate` event](crate::Event::LobbyUpdate) as that event only
    /// fires for metadata changes on the lobby itself, not its members.
    #[serde(default)]
    pub members: Vec<LobbyMember>,
    /// A set of key value pairs to add arbitrary metadata to the lobby.
    pub metadata: Metadata,
    /// The id of the user who owns this lobby.
    pub owner_id: UserId,
    /// The Discord region that the lobby is located in.
    pub region: Region,
    /// The secret required for other users to be able to join this lobby. This
    /// is autogenerated by Discord itself, unlike activity secrets.
    pub secret: String,
    /// Whether the lobby is public or private.
    #[serde(rename = "type")]
    pub kind: LobbyKind,
    #[serde(default)]
    pub voice_states: Vec<VoiceState>,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct LobbyMember {
    pub metadata: Metadata,
    pub user: crate::user::User,
    #[serde(skip)]
    pub speaking: bool,
}

/// Argument used to create or modify a [`Lobby`]
#[derive(Serialize, Clone)]
pub struct LobbyArgs {
    /// The id for a lobby, only set when modifying
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<LobbyId>,
    /// The max capacity of the lobby
    capacity: u32,
    /// If the lobby is public or private
    #[serde(rename = "type")]
    kind: LobbyKind,
    /// Whether or not the lobby can be joined
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
    /// The ID of the user to make the owner, only set when modifying
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<UserId>,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    metadata: Metadata,
}

impl LobbyArgs {
    pub fn modify(self, lobby: &mut Lobby) {
        lobby.capacity = self.capacity;
        lobby.kind = self.kind;
        lobby.locked = self.locked.unwrap_or(false);
        if let Some(owner) = self.owner_id {
            lobby.owner_id = owner;
        }
        lobby.metadata = self.metadata;
    }
}

/// Supplies the same defaults as those that Discord (currently) sets if the any
/// of the arguments are not specified, to protect from behavior changes in
/// Discord in the future
impl Default for LobbyArgs {
    fn default() -> Self {
        Self {
            id: None,
            capacity: 16,
            kind: LobbyKind::Private,
            locked: None,
            owner_id: None,
            metadata: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct CreateLobbyBuilder {
    inner: LobbyArgs,
}

impl CreateLobbyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn capacity(mut self, capacity: Option<std::num::NonZeroU32>) -> Self {
        self.inner.capacity = capacity.map_or(16, |cap| cap.get());
        self
    }

    #[inline]
    pub fn kind(mut self, kind: LobbyKind) -> Self {
        self.inner.kind = kind;
        self
    }

    #[inline]
    pub fn locked(mut self, locked: bool) -> Self {
        self.inner.locked = Some(locked);
        self
    }

    #[inline]
    pub fn add_metadata(mut self, md: impl IntoIterator<Item = (String, String)>) -> Self {
        self.inner.metadata.extend(md);
        self
    }
}

pub struct UpdateLobbyBuilder {
    inner: LobbyArgs,
}

impl UpdateLobbyBuilder {
    pub fn new(to_update: &Lobby) -> Self {
        Self {
            inner: LobbyArgs {
                id: Some(to_update.id),
                capacity: to_update.capacity,
                kind: to_update.kind,
                locked: if to_update.locked { Some(true) } else { None },
                owner_id: Some(to_update.owner_id),
                metadata: to_update.metadata.clone(),
            },
        }
    }

    #[inline]
    pub fn capacity(mut self, capacity: Option<std::num::NonZeroU32>) -> Self {
        self.inner.capacity = capacity.map_or(16, |cap| cap.get());
        self
    }

    #[inline]
    pub fn kind(mut self, kind: LobbyKind) -> Self {
        self.inner.kind = kind;
        self
    }

    #[inline]
    pub fn locked(mut self, locked: bool) -> Self {
        self.inner.locked = Some(locked);
        self
    }

    #[inline]
    pub fn owner(mut self, owner: Option<UserId>) -> Self {
        self.inner.owner_id = owner;
        self
    }

    #[inline]
    pub fn add_metadata(mut self, md: impl IntoIterator<Item = (String, String)>) -> Self {
        self.inner.metadata.extend(md);
        self
    }

    #[inline]
    pub fn delete_metadata<'key>(mut self, to_remove: impl IntoIterator<Item = &'key str>) -> Self {
        for key in to_remove {
            self.inner.metadata.remove(key);
        }
        self
    }
}

#[derive(Serialize)]
pub struct ConnectLobby {
    pub id: LobbyId,
    pub secret: String,
}

impl<'s> std::convert::TryFrom<&'s str> for ConnectLobby {
    type Error = Error;

    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        s.find(':')
            .and_then(|sep| {
                let id = s[..sep].parse().ok()?;
                let secret = s[sep + 1..].to_owned();

                Some(Self { id, secret })
            })
            .ok_or(Error::NonCanonicalLobbyActivitySecret)
    }
}

/// A message sent by a user to a lobby
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LobbyMessage {
    Binary(Vec<u8>),
    Text(String),
}

impl LobbyMessage {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn binary(bin: impl Into<Vec<u8>>) -> Self {
        Self::Binary(bin.into())
    }
}

impl Serialize for LobbyMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Binary(bin) => {
                let mut data = String::from("data:text/plain;base64,");
                base64::encode_config_buf(bin, base64::STANDARD_NO_PAD, &mut data);

                serializer.serialize_str(&data)
            }
            Self::Text(text) => serializer.serialize_str(text),
        }
    }
}

impl<'de> Deserialize<'de> for LobbyMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use std::fmt;

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = LobbyMessage;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(match v.strip_prefix("data:text/plain;base64,") {
                    Some(encoded) => {
                        let bin = base64::decode_config(encoded, base64::STANDARD_NO_PAD)
                            .map_err(de::Error::custom)?;
                        LobbyMessage::Binary(bin)
                    }
                    None => LobbyMessage::Text(v.to_owned()),
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

/// Used by different command types when performing an action on a specific lobby
#[derive(Serialize)]
struct LobbyAction {
    id: LobbyId,
}

impl crate::Discord {
    /// Creates a new [`Lobby`], automatically joining the current
    /// [`User`](crate::user::User) and making them the owner of the [`Lobby`].
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#createlobby)
    pub async fn create_lobby(&self, args: CreateLobbyBuilder) -> Result<Lobby, Error> {
        let rx = self.send_rpc(CommandKind::CreateLobby, args.inner)?;

        handle_response!(rx, Command::CreateLobby(lobby) => {
            Ok(lobby)
        })
    }

    /// Updates a lobby.
    ///
    /// # Errors
    ///
    /// This call has a rate limit of 10 updates per 5 seconds. If you fear you
    /// might hit that, it may be a good idea to batch your lobby updates into
    /// transactions.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#updatelobby)
    pub async fn update_lobby(&self, args: UpdateLobbyBuilder) -> Result<LobbyArgs, Error> {
        // The response for the lobby update unfortunately doesn't return any
        // actual data for the lobby, so we store the new state and set it once
        // Discord responds to the update, but only the metadata pieces that can
        // be modified by the update, so no changes to members or their metadata
        let update = args.inner.clone();
        let rx = self.send_rpc(CommandKind::UpdateLobby, args.inner)?;

        handle_response!(rx, Command::UpdateLobby => {
            Ok(update)
        })
    }

    /// Deletes the specified lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#deletelobby)
    pub async fn delete_lobby(&self, id: LobbyId) -> Result<(), Error> {
        let rx = self.send_rpc(CommandKind::DeleteLobby, LobbyAction { id })?;

        handle_response!(rx, Command::DeleteLobby => {
            Ok(())
        })
    }

    /// Connects to the specified lobby, which comprises 2 pieces of information,
    /// the lobby identifier, and the lobby secret.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#connectlobby)
    pub async fn connect_lobby(&self, lobby: ConnectLobby) -> Result<Lobby, Error> {
        let rx = self.send_rpc(CommandKind::ConnectToLobby, lobby)?;

        handle_response!(rx, Command::ConnectToLobby(lobby) => {
            Ok(lobby)
        })
    }

    /// Disconnects the current user from a lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#disconnectlobby)
    pub async fn disconnect_lobby(&self, id: LobbyId) -> Result<(), Error> {
        let rx = self.send_rpc(CommandKind::DisconnectFromLobby, LobbyAction { id })?;

        handle_response!(rx, Command::DisconnectFromLobby => {
            Ok(())
        })
    }

    /// Sends a message to the lobby on behalf of the current user. The
    ///
    /// # Errors
    ///
    /// You must be connected to the lobby you are messaging.
    /// This method has a rate limit of 10 messages per 5 seconds.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#sendlobbymessage)
    pub async fn send_lobby_message(
        &self,
        lobby_id: LobbyId,
        data: LobbyMessage,
    ) -> Result<(), Error> {
        #[derive(Serialize)]
        struct SendToLobby {
            lobby_id: LobbyId,
            data: LobbyMessage,
        }

        let rx = self.send_rpc(CommandKind::SendToLobby, SendToLobby { lobby_id, data })?;

        handle_response!(rx, Command::SendToLobby => {
            Ok(())
        })
    }

    /// Connects to the voice channel of the specified lobby.
    ///
    /// # Errors
    ///
    /// The user must be connected to the specified lobby.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#connectvoice)
    pub async fn connect_lobby_voice(&self, id: LobbyId) -> Result<(), Error> {
        let rx = self.send_rpc(CommandKind::ConnectToLobbyVoice, LobbyAction { id })?;

        handle_response!(rx, Command::ConnectToLobbyVoice => {
            Ok(())
        })
    }

    /// Disconnects from the voice channel of the specified lobby.
    ///
    /// # Errors
    ///
    /// The user must be connected to the specified lobby, and be connected to
    /// the voice channel already
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#disconnectvoice)
    pub async fn disconnect_lobby_voice(&self, id: LobbyId) -> Result<(), Error> {
        let rx = self.send_rpc(CommandKind::DisconnectFromLobbyVoice, LobbyAction { id })?;

        handle_response!(rx, Command::DisconnectFromLobbyVoice => {
            Ok(())
        })
    }

    /// Updates the metadata for the specified lobby member.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#updatemember)
    pub async fn update_lobby_member(
        &self,
        lobby_id: LobbyId,
        user_id: UserId,
        metadata: Metadata,
    ) -> Result<(), Error> {
        #[derive(Serialize)]
        struct UpdateMember {
            lobby_id: LobbyId,
            user_id: UserId,
            metadata: Metadata,
        }

        let rx = self.send_rpc(
            CommandKind::UpdateLobbyMember,
            UpdateMember {
                lobby_id,
                user_id,
                metadata,
            },
        )?;

        handle_response!(rx, Command::UpdateLobbyMember => {
            Ok(())
        })
    }
}
