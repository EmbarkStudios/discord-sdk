use serde::{Deserialize, Serialize};

pub mod events;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputMode {
    VoiceActivity,
    PushToTalk { shortcut: String },
}

impl crate::Discord {}
