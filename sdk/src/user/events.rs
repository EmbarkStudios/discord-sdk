use super::*;

#[derive(Deserialize, Debug)]
pub struct ConnectEvent {
    /// The protocol version, we only support v1, which is fine since that is
    /// (currently) the only version
    #[serde(rename = "v")]
    pub version: u32,
    pub config: crate::types::DiscordConfig,
    /// The user that is logged into the Discord application we connected to
    #[serde(deserialize_with = "crate::user::de_user")]
    pub user: User,
}

#[derive(Deserialize, Debug)]
pub struct UpdateEvent {
    /// The user that is logged into the Discord application we connected to
    #[serde(flatten, deserialize_with = "crate::user::de_user")]
    pub user: User,
}

#[derive(Debug)]
pub struct DisconnectEvent {
    pub reason: crate::Error,
}

#[derive(Debug)]
pub enum UserEvent {
    Connect(ConnectEvent),
    Disconnect(DisconnectEvent),
    Update(UpdateEvent),
}
