//! Provides types and functionality for [Activities](https://discord.com/developers/docs/game-sdk/activities)
//! , also known as Rich Presence

pub mod events;

use crate::{user::UserId, Command, CommandKind, Error};
use serde::{Deserialize, Serialize};

/// A party is a uniquely identified group of users, but Discord doesn't really
/// provide much on top of this
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activityparty-struct)
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Party {
    /// A unique identifier for this party
    pub id: String,
    /// Info about the size of the party
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<PartyPrivacy>,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq, Debug, Copy, Clone,
)]
#[repr(u8)]
pub enum PartyPrivacy {
    Private = 0,
    Public = 1,
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

impl IntoTimestamp for time::OffsetDateTime {
    fn into_timestamp(self) -> i64 {
        self.unix_timestamp()
    }
}

impl IntoTimestamp for i64 {
    fn into_timestamp(self) -> i64 {
        self
    }
}

/// The custom art assets to be used in the user's profile when the activity
/// is set. These assets need to be already uploaded to Discord in the application's
/// developer settings.
///
/// [Tips](https://discord.com/developers/docs/rich-presence/best-practices#have-interesting-expressive-art)
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
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
    #[inline]
    fn validate_key(key: &str) -> bool {
        key.len() <= 32 || key.starts_with("http://") || key.starts_with("https://")
    }

    /// Sets the large image and optional text to use for the rich presence profile
    ///
    /// Key images are limited to 32 bytes on the server, and any keys over that are
    /// discarded, however, URL-proxied keys have no such limit. The image text is
    /// limited to 128 bytes and will be truncated if longer than that.
    pub fn large(mut self, key: impl Into<String>, text: Option<impl Into<String>>) -> Self {
        let key = key.into();
        if !Self::validate_key(&key) {
            tracing::warn!("Large Image Key '{key}' is invalid, disregarding");
            return self;
        }

        self.large_image = Some(key);
        self.large_text = truncate(text, "Large Image Text");
        self
    }

    /// Sets the small image and optional text to use for the rich presence profile
    ///
    /// Key images are limited to 32 bytes on the server, and any keys over that are
    /// discarded, however, URL-proxied keys have no such limit. The image text is
    /// limited to 128 bytes and will be truncated if longer than that.
    pub fn small(mut self, key: impl Into<String>, text: Option<impl Into<String>>) -> Self {
        let key = key.into();
        if !Self::validate_key(&key) {
            tracing::warn!("Small Image Key '{key}' is invalid, disregarding");
            return self;
        }

        self.small_image = Some(key);
        self.small_text = truncate(text, "Small Image Text");
        self
    }
}

/// The start and end timestamp of the activity. These are unix timestamps.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activitytimestamps-struct)
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Timestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq, Debug, Copy, Clone,
)]
#[repr(u8)]
pub enum ActivityKind {
    Playing = 0,
    Streaming = 1,
    Listening = 2,
    Watching = 3,
    Custom = 4,
    Competing = 5,
}

impl Default for ActivityKind {
    fn default() -> Self {
        Self::Playing
    }
}

/// The activity kinds you can invite a [`User`](crate::user::User) to engage in.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activityactiontype-enum)
#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq, Debug, Copy, Clone,
)]
#[repr(u8)]
pub enum ActivityActionKind {
    /// Invites the user to join your game
    Join = 1,
    /// Invites the user to spectate your game
    Spectate = 2,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct ActivityInvite {
    /// The user that invited the current user to the activity
    pub user: crate::user::User,
    /// The activity the invite is for
    pub activity: InviteActivity,
    /// The kind of activity the invite is for
    #[serde(rename = "type")]
    pub kind: ActivityActionKind,
    /// I think this is the unique identifier for the channel the invite
    /// was sent to, which is (always?) the private channel between the
    /// 2 users
    pub channel_id: crate::types::ChannelId,
    /// The unique message identifier for the invite
    pub message_id: crate::types::MessageId,
}

/// The reply to send to the [`User`](crate::user::User) who sent a join request.
/// Note that the actual values shown in the API docs are irrelevant as the reply
/// on the wire is actually just a different command kind.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activityjoinrequestreply-enum)
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum JoinRequestReply {
    /// Rejects the join request
    No,
    /// Accepts the join request
    Yes,
    /// Ignores the join request. This is semantically no different from [`No`](Self::No),
    /// at least in the current state of the Discord API
    Ignore,
}

impl From<bool> for JoinRequestReply {
    fn from(b: bool) -> Self {
        if b {
            Self::Yes
        } else {
            Self::No
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
    /// Defines clickable buttons in the activity **OR** secrets  for joining and spectating the
    /// player's game.
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub buttons_or_secrets: Option<ButtonsOrSecrets>,
    #[serde(skip_serializing, rename = "type")]
    pub kind: ActivityKind,
    #[serde(default)]
    /// Whether this activity is an instanced context, like a match
    pub instance: bool,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct InviteActivity {
    /// The unique identifier for the activity
    pub session_id: String,
    /// The timestamp the activity was created
    #[serde(skip_serializing, with = "crate::util::datetime_opt")]
    pub created_at: Option<time::OffsetDateTime>,
    /// The usual activity data
    #[serde(flatten)]
    pub details: Activity,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SetActivity {
    #[serde(flatten)]
    activity: Activity,
    /// The name of the application
    name: Option<String>,
    #[serde(deserialize_with = "crate::util::string::deserialize_opt")]
    application_id: Option<crate::AppId>,
}

/// Secret passwords for joining and spectating the player's game
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
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

/// A clickable button underneath the activity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Button {
    pub label: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ButtonKind {
    Label(String),
    Link(Button),
}

#[derive(Serialize, Debug)]
pub struct ActivityArgs {
    pid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<Activity>,
}

impl Default for ActivityArgs {
    fn default() -> Self {
        Self {
            pid: std::process::id(),
            activity: None,
        }
    }
}

impl From<ActivityBuilder> for ActivityArgs {
    #[inline]
    fn from(ab: ActivityBuilder) -> Self {
        ab.inner
    }
}

#[derive(Default, Debug)]
pub struct ActivityBuilder {
    pub(crate) inner: ActivityArgs,
}

impl ActivityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn with_pid(pid: u32) -> Self {
        Self {
            inner: ActivityArgs {
                pid,
                activity: None,
            },
        }
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

    /// What the player is doing, eg. "Exploring the Wilds of Outland".
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

    /// Set the start timestamp for the activity. If only the start is set, Discord will display `XX:XX elapsed`
    pub fn start_timestamp(mut self, timestamp: impl IntoTimestamp) -> Self {
        match &mut self.inner.activity {
            // Modify an existing activity and add a start timestamp
            Some(activity) => {
                match &mut activity.timestamps {
                    // Add a start timestamp to an existing timestamp object
                    Some(timestamps) => {
                        timestamps.start = Some(timestamp.into_timestamp());
                    }

                    // Create a new timestamp object and set its start
                    None => {
                        activity.timestamps = Some(Timestamps {
                            start: Some(timestamp.into_timestamp()),
                            end: None,
                        });
                    }
                }
            }

            // Init a new activity with only a start timestamp
            None => {
                self.inner.activity = Some(Activity {
                    timestamps: Some(Timestamps {
                        start: Some(timestamp.into_timestamp()),
                        end: None,
                    }),
                    ..Default::default()
                });
            }
        }

        self
    }

    /// Set the end timestamp for the activity. If only the end is set, Discord will display `XX:XX left`
    pub fn end_timestamp(mut self, timestamp: impl IntoTimestamp) -> Self {
        match &mut self.inner.activity {
            // Modify an existing activity and add a start timestamp
            Some(activity) => {
                match &mut activity.timestamps {
                    // Add an end timestamp to an existing timestamp object
                    // Only done if the end is after the start
                    Some(timestamps) => {
                        let timestamp = timestamp.into_timestamp();
                        let start = timestamps.start.unwrap_or(0);

                        if start > timestamp {
                            tracing::warn!(
                                "End timestamp must be greater than the start timestamp"
                            );
                        } else {
                            timestamps.end = Some(timestamp.into_timestamp());
                        }
                    }

                    // Create a new timestamp object and set its end
                    None => {
                        activity.timestamps = Some(Timestamps {
                            start: None,
                            end: Some(timestamp.into_timestamp()),
                        });
                    }
                }
            }

            // Init a new activity with only an end timestamp
            None => {
                self.inner.activity = Some(Activity {
                    timestamps: Some(Timestamps {
                        start: None,
                        end: Some(timestamp.into_timestamp()),
                    }),
                    ..Default::default()
                });
            }
        }

        self
    }

    /// The start and end of a "game" or "session".
    pub fn timestamps(
        mut self,
        start: Option<impl IntoTimestamp>,
        end: Option<impl IntoTimestamp>,
    ) -> Self {
        if let Some(st) = start {
            self = self.start_timestamp(st);
        }
        if let Some(et) = end {
            self = self.end_timestamp(et);
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

    /// Adds up to two buttons with a label and a link other users can click on
    ///
    /// Overwrites any secrets already set in the activity.
    pub fn button(mut self, button: Button) -> Self {
        let button = ButtonKind::Link(button);
        match &mut self.inner.activity {
            Some(Activity {
                buttons_or_secrets, ..
            }) => match buttons_or_secrets {
                Some(ButtonsOrSecrets::Buttons { buttons }) => buttons.push(button),
                buttons_or_secrets => {
                    *buttons_or_secrets = Some(ButtonsOrSecrets::Buttons {
                        buttons: vec![button],
                    });
                }
            },
            None => {
                self.inner.activity = Some(Activity {
                    buttons_or_secrets: Some(ButtonsOrSecrets::Buttons {
                        buttons: vec![button],
                    }),
                    ..Default::default()
                });
            }
        }
        self
    }
    /// Sets secrets, allowing other player's to join or spectate the player's
    /// game
    ///
    /// Overwrites any buttons already set in the activity.
    pub fn secrets(mut self, secrets: Secrets) -> Self {
        if secrets.join.is_none() && secrets.r#match.is_none() && secrets.spectate.is_none() {
            return self;
        }

        match &mut self.inner.activity {
            Some(activity) => {
                activity.buttons_or_secrets = Some(ButtonsOrSecrets::Secrets { secrets });
            }
            None => {
                self.inner.activity = Some(Activity {
                    buttons_or_secrets: Some(ButtonsOrSecrets::Secrets { secrets }),
                    ..Default::default()
                });
            }
        }
        self
    }
}

impl crate::Discord {
    /// Sets the current [`User's`](crate::user::User) presence in Discord to a
    /// new activity.
    ///
    /// # Errors
    /// This has a rate limit of 5 updates per 20 seconds.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#updateactivity)
    pub async fn update_activity(
        &self,
        activity: impl Into<ActivityArgs>,
    ) -> Result<Option<Activity>, Error> {
        let rx = self.send_rpc(CommandKind::SetActivity, activity.into())?;

        // TODO: Keep track of the last set activity and send it immediately if
        // the connection to Discord is lost then reestablished?
        handle_response!(rx, Command::SetActivity(sa) => {
            Ok(sa.map(|sa| sa.activity))
        })
    }

    /// Invites the specified [`User`](crate::user::User) to join the current
    /// user's game.
    ///
    /// # Errors
    /// The current [`User`](crate::user::User) must have their presence updated
    /// with all of the [required fields](https://discord.com/developers/docs/game-sdk/activities#activity-action-field-requirements)
    /// otherwise this call will fail.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#sendinvite)
    pub async fn invite_user(
        &self,
        user_id: UserId,
        message: impl Into<String>,
        kind: ActivityActionKind,
    ) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Invite {
            pid: u32,
            user_id: UserId,
            content: String,
            #[serde(rename = "type")]
            kind: ActivityActionKind,
        }

        let rx = self.send_rpc(
            CommandKind::ActivityInviteUser,
            Invite {
                pid: std::process::id(),
                user_id,
                content: message.into(),
                kind,
            },
        )?;

        handle_response!(rx, Command::ActivityInviteUser => {
            Ok(())
        })
    }

    /// Accepts the invite to another user's activity.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#acceptinvite)
    pub async fn accept_invite(&self, invite: &impl AsRef<ActivityInvite>) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Accept<'stack> {
            user_id: UserId,
            #[serde(rename = "type")]
            kind: ActivityActionKind,
            session_id: &'stack str,
            channel_id: crate::types::ChannelId,
            message_id: crate::types::MessageId,
        }

        let invite = invite.as_ref();

        let rx = self.send_rpc(
            CommandKind::AcceptActivityInvite,
            Accept {
                user_id: invite.user.id,
                kind: invite.kind,
                session_id: &invite.activity.session_id,
                channel_id: invite.channel_id,
                message_id: invite.message_id,
            },
        )?;

        handle_response!(rx, Command::AcceptActivityInvite => {
            Ok(())
        })
    }

    /// Clears the rich presence for the logged in [`User`](crate::user::User).
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#clearactivity)
    pub async fn clear_activity(&self) -> Result<Option<Activity>, Error> {
        let rx = self.send_rpc(CommandKind::SetActivity, ActivityArgs::default())?;

        handle_response!(rx, Command::SetActivity(sa) => {
            Ok(sa.map(|sa| sa.activity))
        })
    }

    /// Sends a reply to an [Ask to Join](crate::Event::ActivityJoinRequest) request.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/activities#sendrequestreply)
    pub async fn send_join_request_reply(
        &self,
        user_id: UserId,
        reply: impl Into<JoinRequestReply>,
    ) -> Result<(), Error> {
        let reply = reply.into();

        let kind = match reply {
            JoinRequestReply::Yes => CommandKind::SendActivityJoinInvite,
            JoinRequestReply::No | JoinRequestReply::Ignore => {
                CommandKind::CloseActivityJoinRequest
            }
        };

        #[derive(Serialize)]
        struct JoinReply {
            user_id: UserId,
        }

        let rx = self.send_rpc(kind, JoinReply { user_id })?;

        match reply {
            JoinRequestReply::Yes => {
                handle_response!(rx, Command::SendActivityJoinInvite => {
                    Ok(())
                })
            }
            JoinRequestReply::No | JoinRequestReply::Ignore => {
                handle_response!(rx, Command::CloseActivityJoinRequest => {
                    Ok(())
                })
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ButtonsOrSecrets {
    Buttons { buttons: Vec<ButtonKind> },
    Secrets { secrets: Secrets },
}

/// All strings in the rich presence info have limits enforced in discord itself
/// so we just truncate them manually client side to avoid sending more data
#[inline]
fn truncate(text: Option<impl Into<String>>, name: &str) -> Option<String> {
    text.and_then(|text| {
        let mut text = text.into();
        if text.len() > 128 {
            tracing::warn!("{} '{}' is too long and will be truncated", name, text);
            text.truncate(128);
        }

        // Ensure the strings don't have just whitespace, as they are also not
        // allowed
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde() {
        let args: ActivityArgs = ActivityBuilder::with_pid(9999)
            .details("deetz")
            // This won't be set, as empty state is not allowed
            .state("       ")
            .start_timestamp(1628629161811)
            .end_timestamp(1628629327961)
            .party(
                "parrrrty",
                std::num::NonZeroU32::new(1),
                std::num::NonZeroU32::new(2),
                PartyPrivacy::Private,
            )
            .secrets(Secrets {
                join: Some("sekret".to_owned()),
                ..Default::default()
            })
            .into();

        let cmd = crate::proto::Rpc {
            cmd: CommandKind::SetActivity,
            nonce: 2.to_string(),
            evt: None,
            args: Some(args),
        };

        insta::assert_json_snapshot!(cmd);
    }

    #[test]
    fn asset_keys() {
        assert!(Assets::validate_key("tiny_key"));
        assert!(Assets::validate_key("_-_thirtytwocharacterassetkey_-_"));
        assert!(!Assets::validate_key("_-_thirtythreecharacterassetkey_-"));
        assert!(Assets::validate_key("http://shortboi.com/image"));
        assert!(Assets::validate_key(
            "https://superlongboibutthatsokbecauseitshttps.com/image"
        ));
    }
}
