#[derive(Debug, serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(crate) struct VoiceSettingsUpdateEvent {
    input_mode: super::InputMode,
    local_mute: Vec<crate::user::UserId>,
    local_volumes: std::collections::BTreeMap<crate::user::UserId, u8>,
    self_mute: bool,
    self_deaf: bool,
}
