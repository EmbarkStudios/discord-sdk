use crate::{
    types::{Command, CommandKind, Snowflake, UserId},
    AppId, DiscordErr, Error,
};
use serde::{Deserialize, Serialize};

pub type Metadata = std::collections::BTreeMap<String, String>;
pub type LobbyId = Snowflake;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
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
    Sydney,
    UsCentral,
    UsEast,
    UsSouth,
    UsWest,
    VipAmsterdam,
    VipUsEast,
    VipUsWest,
}

#[derive(Copy, Clone, Debug)]
pub enum LobbyType {
    Private = 1,
    Public = 2,
}

impl Serialize for LobbyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(match self {
            Self::Private => 1,
            Self::Public => 2,
        })
    }
}

impl<'de> serde::Deserialize<'de> for LobbyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = LobbyType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an integer of 1 or 2")
            }

            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    1 => Ok(LobbyType::Private),
                    2 => Ok(LobbyType::Public),
                    other => Err(serde::de::Error::custom(format!(
                        "unknown lobby type: {}",
                        other
                    ))),
                }
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u64(value as u64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    1 => Ok(LobbyType::Private),
                    2 => Ok(LobbyType::Public),
                    other => Err(serde::de::Error::custom(format!(
                        "unknown lobby type: {}",
                        other
                    ))),
                }
            }
        }

        deserializer.deserialize_i32(Visitor)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Lobby {
    pub application_id: AppId,
    pub capacity: u32,
    pub id: LobbyId,
    pub locked: bool,
    pub members: Vec<LobbyMember>,
    pub metadata: Metadata,
    pub owner_id: UserId,
    pub region: Region,
    pub secret: String,
    #[serde(rename = "type")]
    pub kind: LobbyType,
    pub voice_states: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LobbyMember {
    pub metadata: Metadata,
    #[serde(deserialize_with = "crate::types::de_user")]
    pub user: crate::User,
}

/// Argument used to create or modify a [`Lobby`]
#[derive(Serialize, Clone)]
struct LobbyArgs {
    /// The id for a lobby, only set when modifying
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<LobbyId>,
    /// The max capacity of the lobby
    capacity: u32,
    /// If the lobby is public or private
    #[serde(rename = "type")]
    kind: LobbyType,
    /// Whether or not the lobby can be joined
    locked: bool,
    /// The ID of the user to make the owner, only set when modifying
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<UserId>,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    metadata: Metadata,
}

impl LobbyArgs {
    fn modify(self, lobby: &mut Lobby) {
        lobby.capacity = self.capacity;
        lobby.kind = self.kind;
        lobby.locked = self.locked;
        if let Some(owner) = self.owner {
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
            kind: LobbyType::Private,
            locked: false,
            owner: None,
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
    pub fn kind(mut self, kind: LobbyType) -> Self {
        self.inner.kind = kind;
        self
    }

    #[inline]
    pub fn locked(mut self, locked: bool) -> Self {
        self.inner.locked = locked;
        self
    }

    #[inline]
    pub fn add_metadata(mut self, md: impl IntoIterator<Item = (String, String)>) -> Self {
        self.inner.metadata.extend(md);
        self
    }
}

#[derive(Default)]
pub struct UpdateLobbyBuilder {
    inner: LobbyArgs,
}

impl UpdateLobbyBuilder {
    #[inline]
    pub fn capacity(mut self, capacity: Option<std::num::NonZeroU32>) -> Self {
        self.inner.capacity = capacity.map_or(16, |cap| cap.get());
        self
    }

    #[inline]
    pub fn kind(mut self, kind: LobbyType) -> Self {
        self.inner.kind = kind;
        self
    }

    #[inline]
    pub fn locked(mut self, locked: bool) -> Self {
        self.inner.locked = locked;
        self
    }

    #[inline]
    pub fn owner(mut self, owner: Option<UserId>) -> Self {
        self.inner.owner = owner;
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

impl crate::Discord {
    /// Creates a new [`Lobby`], owned by the current [`User`].
    pub async fn create_lobby(&self, args: CreateLobbyBuilder) -> Result<Lobby, Error> {
        let rx = self.send_rpc(CommandKind::CreateLobby, args.inner)?;

        handle_response!(rx, Command::CreateLobby(lobby) => {
            self.owned_lobbies.write().push(lobby.clone());

            Ok(lobby)
        })
    }

    /// Retrieves a builder for the specified lobby to update it. This will fail
    /// if the current [`User`] is not the owner of the lobby.
    pub fn get_lobby_update(&self, lobby_id: LobbyId) -> Result<UpdateLobbyBuilder, Error> {
        self.owned_lobbies
            .read()
            .iter()
            .find_map(|lobby| {
                if lobby.id == lobby_id {
                    let inner = LobbyArgs {
                        id: Some(lobby.id),
                        capacity: lobby.capacity,
                        kind: lobby.kind,
                        locked: lobby.locked,
                        owner: Some(lobby.owner_id),
                        metadata: lobby.metadata.clone(),
                    };

                    Some(UpdateLobbyBuilder { inner })
                } else {
                    None
                }
            })
            .ok_or(Error::Discord(DiscordErr::UnownedLobby(lobby_id)))
    }

    pub async fn update_lobby(&self, args: UpdateLobbyBuilder) -> Result<Lobby, Error> {
        // The response for the lobby update unfortunately doesn't return any
        // actual data for the lobby, so we store the new state and set it once
        // Discord responds to the update, but only the metadata pieces that can
        // be modified by the update, so no changes to members or their metadata
        let update = args.inner.clone();

        let lobby_id = update.id.ok_or(Error::Discord(DiscordErr::UnknownLobby))?;

        let rx = self.send_rpc(CommandKind::UpdateLobby, args.inner)?;

        handle_response!(rx, Command::UpdateLobby => {
            // If ownership was transferred we remove it from the owned list, but
            // _don't_ add it to the searched lobbies since that should be intentional
            // by the user
            let mut ol = self.owned_lobbies.write();

            match ol.iter_mut().position(|lobby| lobby.id == lobby_id) {
                Some(lobby_ind) => {
                    if update.owner != Some(ol[lobby_ind].owner_id) {
                        let mut unowned_lobby = ol.remove(lobby_ind);
                        update.modify(&mut unowned_lobby);

                        Ok(unowned_lobby)
                    } else {
                        let owned_lobby = &mut ol[lobby_ind];
                        update.modify(owned_lobby);
                        Ok(owned_lobby.clone())
                    }
                }
                None => {
                    Err(Error::Discord(DiscordErr::UnownedLobby(lobby_id)))
                }
            }
        })
    }
}
