#[derive(Default, Clone, Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct VoiceSettingsUpdateEvent {
    pub input_mode: Option<super::InputMode>,
    pub local_mute: Vec<crate::user::UserId>,
    pub local_volumes: std::collections::BTreeMap<crate::user::UserId, u8>,
    pub self_mute: bool,
    pub self_deaf: bool,
}

#[derive(Debug, Clone)]
pub enum VoiceEvent {
    /// An actual refresh event from Discord which we use as a source of truth
    Refresh(VoiceSettingsUpdateEvent),
}
