pub mod events;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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

#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LobbyKind {
    Private = 1,
    Public = 2,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Lobby {
    /// The unique identifier for the lobby.
    pub id: LobbyId,
    /// The maximum number of users that can join the lobby.
    pub capacity: u32,
    /// Whether new members can join the lobby.
    pub locked: bool,
    /// The users and attached metadata that are actually present in the lobby.
    /// This list will be empty if this lobby is deserialized from a
    /// [`Event::LobbyUpdate`] as that event only fires for metadata changes on
    /// the lobby itself, not its members.
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
    /// I've never seen this filled out so I actually have no idea what the data
    /// here is
    #[serde(default)]
    pub voice_states: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LobbyMember {
    pub metadata: Metadata,
    #[serde(deserialize_with = "crate::user::de_user")]
    pub user: crate::user::User,
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
    fn modify(self, lobby: &mut Lobby) {
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

/// The logical comparison to use when comparing the value of the filter key in
/// the lobby metadata against the value provided to compare it against
///
/// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#data-models-lobbysearchcomparison-enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum LobbySearchComparison {
    LessThanOrEqual = -2,
    LessThan = -1,
    Equal = 0,
    GreaterThan = 1,
    GreaterThanOrEqual = 2,
    NotEqual = 3,
}

/// The search distance from the current user's region, the [`LobbySearchDistance::Default`]
/// is to search in the current user's region and adjacent regions.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#data-models-lobbysearchdistance-enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LobbySearchDistance {
    /// Within the same region
    Local = 0,
    /// Within the same and adjacent regions
    Default = 1,
    /// Far distances, like US to EU
    Extended = 2,
    /// All regions
    Global = 3,
}

impl Default for LobbySearchDistance {
    fn default() -> Self {
        Self::Default
    }
}

/// Determines how the search value is cast before comparison
///
/// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#data-models-lobbysearchcast-enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LobbySearchCast {
    String = 1,
    Number = 2,
}

#[derive(Serialize)]
pub struct SearchFilter {
    key: String,
    comparison: LobbySearchComparison,
    cast: LobbySearchCast,
    value: String,
}

#[derive(Serialize)]
pub struct SearchSort {
    key: String,
    cast: LobbySearchCast,
    near_value: String,
}

pub enum SearchKey<'md> {
    /// The user id of the owner of the lobby
    OwnerId,
    /// The maximum capacity of the lobby
    Capacity,
    /// The number of available slots in the lobby
    Slots,
    /// A metadata key name
    Metadata(&'md str),
}

impl<'md> From<&'md str> for SearchKey<'md> {
    fn from(key: &'md str) -> Self {
        Self::Metadata(key)
    }
}

use std::fmt;

impl<'md> fmt::Display for SearchKey<'md> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerId => f.write_str("owner_id"),
            Self::Capacity => f.write_str("capacity"),
            Self::Slots => f.write_str("slots"),
            Self::Metadata(key) => write!(f, "metadata.{}", key),
        }
    }
}

pub enum SearchValue {
    String(String),
    Number(String),
}

impl SearchValue {
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    pub fn number<N>(n: N) -> Self
    where
        N: num_traits::PrimInt + fmt::Display,
    {
        Self::Number(n.to_string())
    }

    pub fn cast(&self) -> LobbySearchCast {
        match self {
            Self::String(_) => LobbySearchCast::String,
            Self::Number(_) => LobbySearchCast::Number,
        }
    }
}

impl From<SearchValue> for String {
    fn from(sv: SearchValue) -> Self {
        match sv {
            SearchValue::String(s) | SearchValue::Number(s) => s,
        }
    }
}

/// A query used to [search](https://discord.com/developers/docs/game-sdk/lobbies#search)
/// for lobbies that match a set of criteria.
///
/// By default, this will find a maximum of `25` lobbies in the same or adjacent
/// regions as the current user.
#[derive(Serialize)]
pub struct SearchQuery {
    filter: Vec<SearchFilter>,
    sort: Vec<SearchSort>,
    limit: u32,
    distance: LobbySearchDistance,
}

impl SearchQuery {
    /// Adds a filter to the query which compares the value of the specified key
    /// with the specified comparison against the specified value.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchqueryfilter)
    pub fn add_filter<'md>(
        mut self,
        key: impl Into<SearchKey<'md>>,
        comparison: LobbySearchComparison,
        value: SearchValue,
    ) -> Self {
        self.filter.push(SearchFilter {
            key: key.into().to_string(),
            comparison,
            cast: value.cast(),
            value: value.into(),
        });
        self
    }

    /// Sorts the filtered lobbies based on "near-ness" of the specified key's
    /// value to the specified sort value.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchquerysort)
    pub fn add_sort<'md>(mut self, key: impl Into<SearchKey<'md>>, value: SearchValue) -> Self {
        self.sort.push(SearchSort {
            key: key.into().to_string(),
            cast: value.cast(),
            near_value: value.into(),
        });
        self
    }

    /// Sets the maximum number of lobbies that can be returned by the search.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchquerylimit)
    pub fn limit(mut self, max_results: Option<std::num::NonZeroU32>) -> Self {
        if let Some(mr) = max_results {
            self.limit = mr.get();
        }
        self
    }

    /// Filters lobby results to within certain regions relative to the user's location.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchquerydistance)
    pub fn distance(mut self, distance: LobbySearchDistance) -> Self {
        self.distance = distance;
        self
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            filter: Vec::new(),
            sort: Vec::new(),
            limit: 25,
            distance: Default::default(),
        }
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
#[derive(Debug, PartialEq)]
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
                base64::encode_config_buf(&bin, base64::STANDARD_NO_PAD, &mut data);

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
    /// Creates a new [`Lobby`], automatically joining the current [`User`] and
    /// making them the owner of the [`Lobby`].
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#createlobby)
    pub async fn create_lobby(&self, args: CreateLobbyBuilder) -> Result<Lobby, Error> {
        let rx = self.send_rpc(CommandKind::CreateLobby, args.inner)?;

        handle_response!(rx, Command::CreateLobby(lobby) => {
            *self.owned_lobby.write() = Some(lobby.clone());

            Ok(lobby)
        })
    }

    /// Retrieves a builder for the specified lobby to update it. This will fail
    /// if the current [`User`] is not the owner of the lobby.
    pub fn get_lobby_update(&self, lobby_id: LobbyId) -> Result<UpdateLobbyBuilder, Error> {
        self.owned_lobby
            .read()
            .iter()
            .find_map(|lobby| {
                if lobby.id == lobby_id {
                    let inner = LobbyArgs {
                        id: Some(lobby.id),
                        capacity: lobby.capacity,
                        kind: lobby.kind,
                        locked: if lobby.locked { Some(true) } else { None },
                        owner_id: Some(lobby.owner_id),
                        metadata: lobby.metadata.clone(),
                    };

                    Some(UpdateLobbyBuilder { inner })
                } else {
                    None
                }
            })
            .ok_or(Error::Discord(DiscordErr::UnownedLobby(lobby_id)))
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
            let mut ol = self.owned_lobby.write();

            match &mut *ol {
                Some(lobby) if lobby.id == lobby_id => {
                    // If the owner id is changed we remove it before updating it
                    if update.owner_id != Some(lobby.owner_id) {
                        let mut unowned_lobby = ol.take().unwrap();
                        update.modify(&mut unowned_lobby);
                        Ok(unowned_lobby)
                    } else {
                        update.modify(lobby);
                        Ok(lobby.clone())
                    }
                }
                _ => Err(Error::Discord(DiscordErr::UnownedLobby(lobby_id)))
            }
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
        let rx = self.send_rpc(CommandKind::ConnectToLobbyVoice, LobbyAction { id })?;

        handle_response!(rx, Command::ConnectToLobbyVoice => {
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

    /// Searches available lobbies based on the search criteria
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#search)
    pub async fn search_lobbies(&self, query: SearchQuery) -> Result<Vec<Lobby>, Error> {
        let rx = self.send_rpc(CommandKind::SearchLobbies, query)?;

        handle_response!(rx, Command::SearchLobbies(lobbies) => {
            *self.searched_lobbies.write() = lobbies.clone();

            Ok(lobbies)
        })
    }
}
