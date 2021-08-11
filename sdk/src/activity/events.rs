use super::*;

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SecretEvent {
    pub secret: String,
}

/// Payload for the event fired when a user "Asks to Join" the current user's game
///
/// [API docs](https://discord.com/developers/docs/game-sdk/activities#onactivityjoinrequest)
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct JoinRequestEvent {
    pub user: crate::user::User,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct InviteEvent(pub std::sync::Arc<crate::activity::ActivityInvite>);

impl AsRef<crate::activity::ActivityInvite> for InviteEvent {
    fn as_ref(&self) -> &crate::activity::ActivityInvite {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum ActivityEvent {
    Join(SecretEvent),
    Spectate(SecretEvent),
    JoinRequest(JoinRequestEvent),
    Invite(InviteEvent),
}
