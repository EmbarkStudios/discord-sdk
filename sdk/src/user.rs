//! Provides types and functionality around [Users](https://discord.com/developers/docs/game-sdk/users)

pub mod events;

use crate::Error;
use serde::Deserialize;
use std::{convert::TryFrom, fmt};

pub type UserId = crate::types::Snowflake;

/// A Discord user.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/users#data-models-user-struct)
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
    /// Whether the user belongs to an OAuth2 application
    pub is_bot: bool,
}

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
    /// Whether the user belongs to an OAuth2 application
    bot: Option<bool>,
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
            is_bot: u.bot.unwrap_or(false),
        })
    }
}
