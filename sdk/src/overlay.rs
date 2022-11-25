//! Provides types and functionality for the Discord [Overlay](https://discord.com/developers/docs/game-sdk/overlay)

pub mod events;

use crate::{Command, CommandKind, Error};
use serde::Serialize;

#[derive(Serialize)]
struct OverlayToggle {
    /// Our process id, this lets Discord know what process it should try
    /// to show the overlay in
    pid: u32,
    #[serde(rename = "locked")]
    visibility: Visibility,
}

impl OverlayToggle {
    fn new(visibility: Visibility) -> Self {
        Self {
            pid: std::process::id(),
            visibility,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Serialize for Visibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(!(*self == Self::Visible))
    }
}

impl<'de> serde::Deserialize<'de> for Visibility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Visibility;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a boolean")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(if value {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                })
            }
        }

        deserializer.deserialize_bool(Visitor)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum InviteAction {
    Join = 1,
    Spectate = 2,
}

#[derive(Serialize)]
pub(crate) struct OverlayPidArgs {
    pid: u32,
}

impl OverlayPidArgs {
    pub(crate) fn new() -> Self {
        Self {
            pid: std::process::id(),
        }
    }
}

impl crate::Discord {
    /// Opens or closes the overlay. If the overlay is not enabled this will
    /// instead focus the Discord app itself.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/overlay#setlocked)
    pub async fn set_overlay_visibility(&self, visibility: Visibility) -> Result<(), Error> {
        let rx = self.send_rpc(
            CommandKind::SetOverlayVisibility,
            OverlayToggle::new(visibility),
        )?;

        handle_response!(rx, Command::SetOverlayVisibility => {
            Ok(())
        })
    }

    /// Opens the overlay modal for sending game invitations to users, channels,
    /// and servers.
    ///
    /// # Errors
    /// If you do not have a valid activity with all the required fields, this
    /// call will error. See
    /// [Activity Action Field Requirements](https://discord.com/developers/docs/game-sdk/activities#activity-action-field-requirements)
    /// for the fields required to have join and spectate invites function properly.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/overlay#openactivityinvite)
    pub async fn open_activity_invite(&self, action: InviteAction) -> Result<(), Error> {
        #[derive(Serialize)]
        struct OpenInviteModal {
            /// Our process id, this lets Discord know what process it should try
            /// to show the overlay in
            pid: u32,
            #[serde(rename = "type")]
            kind: InviteAction,
        }

        let rx = self.send_rpc(
            CommandKind::OpenOverlayActivityInvite,
            OpenInviteModal {
                pid: std::process::id(),
                kind: action,
            },
        )?;

        handle_response!(rx, Command::OpenOverlayActivityInvite => {
            Ok(())
        })
    }

    /// Opens the overlay modal for joining a Discord guild, given its invite code.
    /// Unlike the normal SDK, this method automatically parses the code from
    /// the provided string so you don't need to do it yourself.
    ///
    /// Note that just because the result might be [`Result::Ok`] doesn't
    /// necessarily mean the user accepted the invite.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/overlay#openguildinvite)
    pub async fn open_guild_invite(&self, code: impl AsRef<str>) -> Result<(), Error> {
        let mut code = code.as_ref();

        if let Some(rest) = code.strip_prefix("https://") {
            code = rest;
        }

        if let Some(rest) = code.strip_prefix("discord.gg/") {
            code = rest;
        } else if let Some(rest) = code.strip_prefix("discordapp.com/invite/") {
            code = rest;
        }

        #[derive(Serialize)]
        struct OpenGuildInviteModal<'stack> {
            pid: u32,
            code: &'stack str,
        }

        let rx = self.send_rpc(
            CommandKind::OpenOverlayGuildInvite,
            OpenGuildInviteModal {
                pid: std::process::id(),
                code,
            },
        )?;

        handle_response!(rx, Command::OpenOverlayGuildInvite => {
            Ok(())
        })
    }

    /// Opens the overlay widget for voice settings for the currently connected application.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/overlay#openvoicesettings)
    pub async fn open_voice_settings(&self) -> Result<(), Error> {
        let rx = self.send_rpc(CommandKind::OpenOverlayVoiceSettings, OverlayPidArgs::new())?;

        handle_response!(rx, Command::OpenOverlayVoiceSettings => {
            Ok(())
        })
    }
}
