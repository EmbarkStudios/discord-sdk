use crate::{Error, Lobby};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub type UserId = Snowflake;

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
    pub(crate) message: Option<&'stack str>,
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
        #[serde(deserialize_with = "de_user")]
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
        #[serde(deserialize_with = "de_user")]
        user: User,
    },
    /// Fires when we've done something naughty and Discord is telling us to stop.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord#error-handling)
    Error(ErrorPayload),
    /// Fired when the connection has been interrupted between us and Discord
    #[serde(skip)]
    Disconnected { reason: String },
}

/// The response to an RPC sent by us.
#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Command {
    CreateLobby(Lobby),
    UpdateLobby,
    SetActivity(Option<crate::rich_presence::SetActivity>),
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
    #[serde(deserialize_with = "string::deserialize")]
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
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum EventKind {
    Ready,
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
    Error,
}

/// The reply to send to the [`User`] who sent a join request
#[derive(Copy, Clone)]
pub enum JoinReply {
    /// Allow the user to send a join the local user's session
    Accept,
    /// Disallow the user from joining the local user's session
    Reject,
}

/// The different RPC command types
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandKind {
    /// Dispatch the event specified in "evt".
    Dispatch,
    /// Updates the user's rich presence
    SetActivity,
    /// Subscribes to the event specified in "evt"
    Subscribe,
    /// Unsubscribes from the event specified in "evt"
    Unsubscribe,
    /// RPC sent when the local user has [`JoinReply::Accept`]ed a join request
    SendActivityJoinInvite,
    /// RPC sent when the local user has [`JoinReply::Reject`]ed a join request
    CloseActivityRequest,
    /// RPC sent to create a lobby
    CreateLobby,
    /// RPC sent to modify the mutable properties of a lobby
    UpdateLobby,
}

/// A Discord user
#[derive(Clone)]
pub struct User {
    /// The user's id
    pub id: UserId,
    /// The username
    pub username: String,
    /// The user's unique discriminator (ie. the #<number> after their name) to
    /// disambiguate between users with the same username
    pub discriminator: Option<u32>,
    /// The MD5 hash of the user's avatar
    pub avatar: Option<[u8; 16]>,
}

use std::fmt;

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("discriminator", &self.discriminator)
            .finish()
    }
}

/// Display the name of the user exactly as Discord does, eg `john.smith#1337`.
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.username)?;

        if let Some(disc) = self.discriminator {
            write!(f, "#{}", disc)?;
        }

        Ok(())
    }
}

/// Inner type purely used for deserialization because writing it manually is
/// annoying.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#data-models-user-struct)
#[derive(Deserialize)]
struct DeUser<'u> {
    /// The i64 unique id of the user, but serialized as a string for I guess
    /// backwards compatiblity
    id: Option<UserId>,
    /// The user's username
    username: Option<&'u str>,
    /// A u32 discriminator (serialized as a string, again) to disambiguate
    /// between users with the same username
    discriminator: Option<&'u str>,
    /// A hex-encoded MD5 hash of the user's avatar
    avatar: Option<&'u str>,
}

pub(crate) fn de_user<'de, D>(d: D) -> Result<User, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let u: DeUser<'de> = serde::de::Deserialize::deserialize(d)?;
    User::try_from(u).map_err(serde::de::Error::custom)
}

impl<'de> TryFrom<DeUser<'de>> for User {
    type Error = Error;

    fn try_from(u: DeUser<'de>) -> Result<Self, Self::Error> {
        let id = u.id.ok_or(Error::MissingField("id"))?;
        let username = u
            .username
            .ok_or(Error::MissingField("username"))?
            .to_owned();

        // We _could_ ignore parse failures for this, but that might be confusing
        let discriminator = match u.discriminator {
            Some(d) => Some(
                d.parse()
                    .map_err(|_err| Error::InvalidField("discriminator"))?,
            ),
            None => None,
        };
        // We don't really do anything with this so it's allowed to fail
        let avatar = match u.avatar {
            Some(a) => {
                let avatar = a.strip_prefix("a_").unwrap_or(a);

                if avatar.len() != 32 {
                    None
                } else {
                    let mut md5 = [0u8; 16];
                    let mut valid = true;

                    for (ind, exp) in avatar.as_bytes().chunks(2).enumerate() {
                        let mut cur;

                        match exp[0] {
                            b'A'..=b'F' => cur = exp[0] - b'A' + 10,
                            b'a'..=b'f' => cur = exp[0] - b'a' + 10,
                            b'0'..=b'9' => cur = exp[0] - b'0',
                            c => {
                                tracing::debug!("invalid character '{}' found in avatar", c);
                                valid = false;
                                break;
                            }
                        }

                        cur <<= 4;

                        match exp[1] {
                            b'A'..=b'F' => cur |= exp[1] - b'A' + 10,
                            b'a'..=b'f' => cur |= exp[1] - b'a' + 10,
                            b'0'..=b'9' => cur |= exp[1] - b'0',
                            c => {
                                tracing::debug!("invalid character '{}' found in avatar", c);
                                valid = false;
                                break;
                            }
                        }

                        md5[ind] = cur;
                    }

                    valid.then(|| md5)
                }
            }
            None => None,
        };

        Ok(Self {
            id,
            username,
            discriminator,
            avatar,
        })
    }
}

pub(crate) mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer};

    // pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    // where
    //     T: Display,
    //     S: Serializer,
    // {
    //     serializer.collect_str(value)
    // }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }

    // pub fn serialize_opt<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    // where
    //     T: Display,
    //     S: Serializer,
    // {
    //     serializer.collect_str(value.as_ref().unwrap())
    // }

    pub fn deserialize_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Option::<String>::deserialize(deserializer)? {
            Some(s) => Ok(Some(s.parse().map_err(de::Error::custom)?)),
            None => Ok(None),
        }
    }
}

/// Discord uses [snowflakes](https://discord.com/developers/docs/reference#snowflakes)
/// for most/all of their unique identifiers, including users, lobbies, etc
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Snowflake(u64);

impl Snowflake {
    pub fn timestamp(self) -> chrono::DateTime<chrono::Utc> {
        let millis = self.0.overflowing_shr(22).0 + 1420070400000;
        let ts_seconds = millis / 1000;
        let ts_nanos = (millis % 1000) as u32 * 1000000;

        use chrono::TimeZone;
        chrono::Utc.timestamp(ts_seconds as i64, ts_nanos)
    }
}

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
