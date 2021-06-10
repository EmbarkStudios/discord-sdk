use crate::{
    types::{Command, CommandKind},
    Error,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Party {
    /// A unique identifier for this party
    pub id: String,
    /// Info about the size of the party
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<PartyPrivacy>,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum PartyPrivacy {
    Private,
    Public,
}

pub trait IntoTimestamp {
    fn into_timestamp(self) -> i64;
}

impl IntoTimestamp for std::time::SystemTime {
    fn into_timestamp(self) -> i64 {
        match self.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(dur) => dur.as_secs() as i64,
            Err(_) => 0,
        }
    }
}

impl<Tz: chrono::TimeZone> IntoTimestamp for chrono::DateTime<Tz> {
    fn into_timestamp(self) -> i64 {
        self.timestamp()
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Assets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

impl Assets {
    /// Sets the large image and optional text to use for the rich presence profile
    ///
    /// The key is limited to 32 bytes on the server, so any keys over that size
    /// will be considered invalid and won't be set. The image text is limited
    /// to 128 bytes and will be truncated if longer than that.
    pub fn large(mut self, key: impl Into<String>, text: Option<impl Into<String>>) -> Self {
        let key = key.into();
        if key.len() > 32 {
            tracing::warn!("Large Image Key '{}' is invalid, disregarding", key);
            return self;
        }

        self.large_image = Some(key);
        self.large_text = truncate(text, "Large Image Text");
        self
    }

    /// Sets the small image and optional text to use for the rich presence profile
    ///
    /// The key is limited to 32 bytes on the server, so any keys over that size
    /// will be considered invalid and won't be set. The image text is limited
    /// to 128 bytes and will be truncated if longer than that.
    pub fn small(mut self, key: impl Into<String>, text: Option<impl Into<String>>) -> Self {
        let key = key.into();
        if key.len() > 32 {
            tracing::warn!("Small Image Key '{}' is invalid, disregarding", key);
            return self;
        }

        self.small_image = Some(key);
        self.small_text = truncate(text, "Small Image Text");
        self
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Timestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Activity {
    /// The player's current party status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// What the player is currently doing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Helps create elapsed/remaining timestamps on a player's profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Timestamps>,
    /// Assets to display on the player's profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Assets>,
    /// Information about the player's party
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party: Option<Party>,
    /// Secret passwords for joining and spectating the player's game
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Secrets>,
    #[serde(default)]
    /// Whether this activity is an instanced context, like a match
    pub instance: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetActivity {
    #[serde(flatten)]
    activity: Activity,
    /// The name of the application
    name: Option<String>,
    #[serde(deserialize_with = "crate::types::string::deserialize_opt")]
    application_id: Option<crate::AppId>,
}

/// Secret passwords for joining and spectating the player's game
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Secrets {
    /// Unique hash for the given match context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#match: Option<String>,
    /// Unique hash for chat invites and Ask to Join
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,
    /// Unique hash for Spectate button
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectate: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ActivityArgs {
    pid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    activity: Option<Activity>,
}

impl Default for ActivityArgs {
    fn default() -> Self {
        Self {
            pid: std::process::id(),
            activity: None,
        }
    }
}

#[derive(Default)]
pub struct ActivityBuilder {
    pub(crate) inner: ActivityArgs,
}

impl ActivityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// The user's currenty party status, eg. "Playing Solo".
    ///
    /// Limited to 128 bytes.
    pub fn state(mut self, state: impl Into<String>) -> Self {
        let state = truncate(Some(state), "State");

        match &mut self.inner.activity {
            Some(activity) => activity.state = state,
            None => {
                self.inner.activity = Some(Activity {
                    state,
                    ..Default::default()
                });
            }
        }

        self
    }

    /// What the player is doing, eg. "Exploring the Wild of Outland".
    ///
    /// Limited to 128 bytes.
    pub fn details(mut self, details: impl Into<String>) -> Self {
        let details = truncate(Some(details), "Details");

        match &mut self.inner.activity {
            Some(activity) => activity.details = details,
            None => {
                self.inner.activity = Some(Activity {
                    details,
                    ..Default::default()
                });
            }
        }

        self
    }

    /// The start and optionally end of a "game" or "session".
    pub fn timestamps(
        mut self,
        start: Option<impl IntoTimestamp>,
        end: Option<impl IntoTimestamp>,
    ) -> Self {
        let start = start.map(IntoTimestamp::into_timestamp);
        let end = end.map(IntoTimestamp::into_timestamp);

        let timestamps = match (start, end) {
            (Some(st), Some(et)) => {
                if st >= et {
                    tracing::warn!("End timestamp must be greater than the start timestamp");
                    return self;
                }

                Some(Timestamps { start, end })
            }
            (None, None) => return self,
            _ => Some(Timestamps { start, end }),
        };

        match &mut self.inner.activity {
            Some(activity) => activity.timestamps = timestamps,
            None => {
                self.inner.activity = Some(Activity {
                    timestamps,
                    ..Default::default()
                });
            }
        }

        self
    }

    /// The image assets to use for the rich presence profile
    pub fn assets(mut self, assets: Assets) -> Self {
        if assets.large_image.is_none() && assets.small_image.is_none() {
            return self;
        }

        let assets = Some(assets);

        match &mut self.inner.activity {
            Some(activity) => activity.assets = assets,
            None => {
                self.inner.activity = Some(Activity {
                    assets,
                    ..Default::default()
                });
            }
        }

        self
    }

    /// Sets party details such as size and whether it can be joined by others.
    ///
    /// Note that the party size will only be set if both size and max are provided,
    /// and that the party id is limited to 128 bytes.
    pub fn party(
        mut self,
        id: impl Into<String>,
        current_size: Option<std::num::NonZeroU32>,
        max_size: Option<std::num::NonZeroU32>,
        privacy: PartyPrivacy,
    ) -> Self {
        let id = truncate(Some(id), "Party Id").unwrap();

        let size = match (current_size, max_size) {
            (Some(cur), Some(max)) => {
                let cur = cur.get();
                let max = max.get();

                if cur > max {
                    tracing::warn!(
                        "The current size of the party was larger than the maximum size"
                    );
                    None
                } else {
                    Some((cur, max))
                }
            }
            _ => None,
        };

        let party = Some(Party {
            id,
            size,
            privacy: Some(privacy),
        });

        match &mut self.inner.activity {
            Some(activity) => activity.party = party,
            None => {
                self.inner.activity = Some(Activity {
                    party,
                    ..Default::default()
                });
            }
        }

        self
    }

    /// Whether this activity is an instanced context, like a match
    pub fn instance(mut self, is_instance: bool) -> Self {
        match &mut self.inner.activity {
            Some(activity) => activity.instance = is_instance,
            None => {
                self.inner.activity = Some(Activity {
                    instance: is_instance,
                    ..Default::default()
                });
            }
        }

        self
    }

    /// Sets secrets, allowing other player's to join or spectate the player's
    /// game
    pub fn secrets(mut self, secrets: Secrets) -> Self {
        if secrets.join.is_none() && secrets.r#match.is_none() && secrets.spectate.is_none() {
            return self;
        }

        let secrets = Some(secrets);

        match &mut self.inner.activity {
            Some(activity) => activity.secrets = secrets,
            None => {
                self.inner.activity = Some(Activity {
                    secrets,
                    ..Default::default()
                });
            }
        }

        self
    }
}

impl crate::Discord {
    /// Updates the rich presence for the logged in [`User`].
    pub async fn update_presence(
        &self,
        presence: ActivityBuilder,
    ) -> Result<Option<Activity>, Error> {
        let rx = self.send_rpc(CommandKind::SetActivity, presence.inner)?;

        // TODO: Keep track of the last set activity and send it immediately if
        // the connection to Discord is lost then reestablished?
        handle_response!(rx, Command::SetActivity(sa) => {
            Ok(sa.map(|sa| sa.activity))
        })
    }

    /// Clears the rich presence for the logged in [`User`].
    pub async fn clear_presence(&self) -> Result<Option<Activity>, Error> {
        let rx = self.send_rpc(CommandKind::SetActivity, ActivityArgs::default())?;

        handle_response!(rx, Command::SetActivity(sa) => {
            Ok(sa.map(|sa| sa.activity))
        })
    }
}

/// All strings in the rich presence info have limits enforced in discord itself
/// so we just truncate them manually client side to avoid sending more data
#[inline]
fn truncate(text: Option<impl Into<String>>, name: &str) -> Option<String> {
    text.map(|text| {
        let mut text = text.into();
        if text.len() > 128 {
            tracing::warn!("{} '{}' is too long and will be truncated", name, text);
            text.truncate(128);
        }

        text
    })
}
