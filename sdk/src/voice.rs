use crate::{
    proto::{Command, CommandKind},
    user::UserId,
    Error,
};
use serde::{Deserialize, Serialize};

pub mod events;
pub mod state;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputMode {
    VoiceActivity,
    PushToTalk { shortcut: String },
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
