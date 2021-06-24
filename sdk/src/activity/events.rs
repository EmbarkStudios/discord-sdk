use super::*;

#[derive(Deserialize, Debug, Clone)]
pub struct SecretEvent {
    pub secret: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JoinRequestEvent {
    #[serde(deserialize_with = "crate::user::de_user")]
    pub user: crate::user::User,
}

pub type InviteEvent = std::sync::Arc<crate::activity::ActivityInvite>;

#[derive(Debug, Clone)]
pub enum ActivityEvent {
    Join(SecretEvent),
    Spectate(SecretEvent),
    JoinRequest(JoinRequestEvent),
    Invite(InviteEvent),
}
