//! Provides types and functionality for [Relationships](https://discord.com/developers/docs/game-sdk/relationships)

pub mod events;
pub mod state;

use crate::{user::User, Error};
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;

#[derive(Copy, Clone, Debug, PartialEq, serde_repr::Deserialize_repr)]
#[cfg_attr(test, derive(serde_repr::Serialize_repr))]
#[repr(u8)]
pub enum RelationKind {
    /// User has no intrinsic relationship
    None = 0,
    /// User is a friend
    Friend = 1,
    /// User is blocked
    Blocked = 2,
    /// User has a pending incoming friend request to connected user
    PendingIncoming = 3,
    /// Current user has a pending outgoing friend request to user
    PendingOutgoing = 4,
    /// User is not friends, but interacts with current user often (frequency + recency)
    Implicit = 5,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "snake_case")]
pub enum RelationStatus {
    /// The user is offline
    Offline,
    /// The user is online and active
    Online,
    /// The user is online, but inactive
    Idle,
    /// The user has set their status to Do Not Disturb
    #[serde(rename = "dnd")]
    DoNotDisturb,
}

/// The start and end timestamp of the activity. These are unix timestamps.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#data-models-activitytimestamps-struct)
#[derive(Default, Clone, Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct RelationshipActivityTimestamps {
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "crate::util::datetime_opt",
        default
    )]
    pub start: Option<time::OffsetDateTime>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "crate::util::datetime_opt",
        default
    )]
    pub end: Option<time::OffsetDateTime>,
}

use crate::activity;

#[derive(Default, Clone, Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct RelationshipActivity {
    /// The unique identifier for the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// The timestamp the activity was created
    #[serde(skip_serializing, with = "crate::util::datetime_opt")]
    pub created_at: Option<time::OffsetDateTime>,
    /// The player's current party status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// What the player is currently doing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Helps create elapsed/remaining timestamps on a player's profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<RelationshipActivityTimestamps>,
    /// Assets to display on the player's profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<activity::Assets>,
    /// Information about the player's party
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party: Option<activity::Party>,
    /// Secret passwords for joining and spectating the player's game
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<activity::Secrets>,
    #[serde(rename = "type")]
    pub kind: activity::ActivityKind,
    #[serde(default)]
    /// Whether this activity is an instanced context, like a match
    pub instance: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct RelationshipPresence {
    pub status: RelationStatus,
    pub activity: Option<RelationshipActivity>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct Relationship {
    /// What kind of relationship it is
    #[serde(rename = "type")]
    pub kind: RelationKind,
    pub user: User,
    pub presence: RelationshipPresence,
}

impl crate::Discord {
    /// The regular Game SDK does not really expose this functionality directly,
    /// but rather exposed via the "on refresh" event as described in the
    /// [docs](https://discord.com/developers/docs/game-sdk/relationships#onrefresh).
    ///
    /// Basically, this method should be used to bootstrap the relationships for
    /// the current user, with updates to that list coming via the
    /// [`RelationshipUpdate`](crate::Event::RelationshipUpdate) event
    pub async fn get_relationships(&self) -> Result<Vec<Relationship>, Error> {
        let rx = self.send_rpc(crate::proto::CommandKind::GetRelationships, ())?;

        handle_response!(rx, crate::proto::Command::GetRelationships { relationships } => {
            Ok(relationships)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{activity, proto::event};

    #[test]
    fn deserializes() {
        let event = r#"{"cmd":"DISPATCH","data":{"type":1,"user":{"id":"682969165652689005","username":"jake.shadle","discriminator":"7557","avatar":"15bbd75c8ee6610d045852e7ea998a35","bot":false,"flags":0,"premium_type":0},"presence":{"status":"online","activity":{"created_at":"1632819046295","id":"e92ece5eb4ce629","name":"Ark [dev debug]","timestamps":{"start":"1632819046199"},"type":0}}},"evt":"RELATIONSHIP_UPDATE","nonce":null}"#;

        let update: crate::proto::event::EventFrame =
            serde_json::from_str(event).expect("failed to deserialize");

        insta::assert_json_snapshot!(update);
    }

    #[test]
    fn serde() {
        let eve = event::EventFrame {
            inner: event::Event::RelationshipUpdate(std::sync::Arc::new(Relationship {
                kind: RelationKind::Friend,
                user: User {
                    id: crate::types::Snowflake(123414231424),
                    username: "name".to_owned(),
                    discriminator: Some(52),
                    avatar: Some(crate::user::Avatar([
                        0xf6, 0x2f, 0x2a, 0x75, 0x5c, 0xb1, 0x8c, 0x94, 0xdc, 0x5c, 0xda, 0x94,
                        0x44, 0x10, 0x24, 0xf1,
                    ])),
                    is_bot: false,
                },
                presence: RelationshipPresence {
                    status: RelationStatus::DoNotDisturb,
                    activity: Some(RelationshipActivity {
                        session_id: Some("6bb1ddaea510750e905615286709d632".to_owned()),
                        created_at: Some(crate::util::timestamp(1628629162447)),
                        assets: Some(activity::Assets {
                            large_image: Some(
                                "spotify:ab67616d0000b273d1e326d10706f3d8562d77f8".to_owned(),
                            ),
                            large_text: Some("To the Moon".to_owned()),
                            small_image: None,
                            small_text: None,
                        }),
                        details: Some("To the Moon".to_owned()),
                        instance: false,
                        kind: activity::ActivityKind::Listening,
                        party: Some(activity::Party {
                            id: "spotify: 216453179196440576".to_owned(),
                            size: None,
                            privacy: None,
                        }),
                        secrets: None,
                        state: Some("Rob Curly".to_owned()),
                        timestamps: Some(RelationshipActivityTimestamps {
                            start: Some(crate::util::timestamp(1628629161811)),
                            end: Some(crate::util::timestamp(1628629327961)),
                        }),
                    }),
                },
            })),
        };

        insta::assert_json_snapshot!(eve);
    }
}
