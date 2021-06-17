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
