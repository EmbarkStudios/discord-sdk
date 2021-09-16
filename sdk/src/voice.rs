use crate::{
    proto::{Command, CommandKind},
    user::UserId,
    Error,
};
use serde::{Deserialize, Serialize};

pub mod events;
pub mod state;

#[derive(Clone, Debug)]
pub enum InputMode {
    VoiceActivity,
    PushToTalk { shortcut: String },
}

impl<'de> Deserialize<'de> for InputMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner<'stack> {
            #[serde(rename = "type")]
            kind: &'stack str,
            shortcut: Option<&'stack str>,
        }

        let inner = Inner::<'de>::deserialize(deserializer)?;

        Ok(match inner.kind {
            "VOICE_ACTIVITY" => Self::VoiceActivity,
            "PUSH_TO_TALK" => Self::PushToTalk {
                shortcut: inner.shortcut.unwrap_or_default().to_owned(),
            },
            other => return Err(de::Error::custom(format!("unknown variant '{}'", other))),
        })
    }
}

impl Serialize for InputMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use ser::SerializeStruct;

        let mut state = serializer.serialize_struct("InputMode", 2)?;

        match self {
            Self::VoiceActivity => {
                state.serialize_field("type", "VOICE_ACTIVITY")?;
                // HACK: Discord will give errors if shortcut is not supplied AND it's a string AND it's not empty :(
                state.serialize_field("shortcut", "_")?;
            }
            Self::PushToTalk { shortcut } => {
                state.serialize_field("type", "PUSH_TO_TALK")?;
                state.serialize_field("shortcut", shortcut)?;
            }
        }

        state.end()
    }
}

impl crate::Discord {
    /// Mutes or unmutes the currently connected user.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord-voice#setselfmute)
    pub async fn voice_mute(&self, mute: bool) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Mute {
            self_mute: bool,
        }

        let rx = self.send_rpc(CommandKind::SetVoiceSettings, Mute { self_mute: mute })?;

        handle_response!(rx, Command::SetVoiceSettings => {
            Ok(())
        })
    }

    /// Deafens or undefeans the currently connected user.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord-voice#setselfdeaf)
    pub async fn voice_deafen(&self, deaf: bool) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Deafen {
            self_deaf: bool,
        }

        let rx = self.send_rpc(CommandKind::SetVoiceSettings, Deafen { self_deaf: deaf })?;

        handle_response!(rx, Command::SetVoiceSettings => {
            Ok(())
        })
    }

    /// Sets a new voice input mode for the user. Refer to [Shortcut Keys](
    /// https://discord.com/developers/docs/game-sdk/discord-voice#data-models-shortcut-keys)
    /// for a table of valid values for shortcuts.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord-voice#setinputmode)
    pub async fn voice_set_input_mode(&self, input_mode: InputMode) -> Result<(), Error> {
        #[derive(Serialize)]
        struct SetInputMode {
            input_mode: InputMode,
        }

        let rx = self.send_rpc(CommandKind::SetVoiceSettings, SetInputMode { input_mode })?;

        handle_response!(rx, Command::SetVoiceSettings => {
            Ok(())
        })
    }

    /// Mutes or unmutes the given user for the currently connected user.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord-voice#setlocalmute)
    pub async fn voice_mute_user(&self, user: UserId, mute: bool) -> Result<(), Error> {
        #[derive(Serialize)]
        struct UserMute {
            user_id: UserId,
            mute: bool,
        }

        let rx = self.send_rpc(
            CommandKind::SetUserVoiceSettings,
            UserMute {
                user_id: user,
                mute,
            },
        )?;

        handle_response!(rx, Command::SetUserVoiceSettings => {
            Ok(())
        })
    }

    /// Sets the local volume for a given user. This is the volume level at
    /// which the currently connected users hears the given user speak. Valid
    /// volume values are from 0 to 200, with 100 being the default. Lower than
    /// 100 will be a reduced volume level from default, whereas over 100 will
    /// be a boosted volume level from default.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/discord-voice#setlocalvolume)
    pub async fn voice_set_user_volume(&self, user: UserId, volume: u8) -> Result<(), Error> {
        #[derive(Serialize)]
        struct UserVolume {
            user_id: UserId,
            volume: u8,
        }

        let rx = self.send_rpc(
            CommandKind::SetUserVoiceSettings,
            UserVolume {
                user_id: user,
                volume: std::cmp::min(volume, 200),
            },
        )?;

        handle_response!(rx, Command::SetUserVoiceSettings => {
            Ok(())
        })
    }
}
