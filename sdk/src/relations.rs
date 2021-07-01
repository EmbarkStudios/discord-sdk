pub mod events;
pub mod state;

use crate::{user::User, Error};
use serde::Deserialize;

#[derive(Copy, Clone, Debug, PartialEq, serde_repr::Deserialize_repr)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipPresence {
    pub status: RelationStatus,
    pub activity: Option<crate::activity::Activity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Relationship {
    /// What kind of relationship it is
    #[serde(rename = "type")]
    pub kind: RelationKind,
    #[serde(deserialize_with = "crate::user::de_user")]
    pub user: User,
    pub presence: RelationshipPresence,
}

impl crate::Discord {
    /// The regular Game SDK does not really expose this functionality directly,
    /// but rather exposed via the "on refresh" event as described in the [docs].
    ///
    /// Basically, this method should be used to bootstrap the relationships for
    /// for the user, with updates to that list coming via the
    /// [`RelationshipUpdate`](crate::Event::RelationshipUpdate) event
    pub async fn get_relationships(&self) -> Result<Vec<Relationship>, Error> {
        let rx = self.send_rpc(crate::proto::CommandKind::GetRelationships, ())?;

        handle_response!(rx, crate::proto::Command::GetRelationships { relationships } => {
            Ok(relationships)
        })
    }
}
