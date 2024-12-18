#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("a connection could not be established with Discord")]
    NoConnection,
    #[error("a channel is full and can't receive more messages")]
    ChannelFull,
    #[error("a channel is disconnected and no more messages can be sent")]
    ChannelDisconnected,
    #[error("Discord closed the connection: {0}")]
    Close(String),
    #[error("received an invalid message Discord which indicates the connection is corrupted")]
    CorruptConnection,
    #[error("a message from Discord was missing expected field '{0}'")]
    MissingField(&'static str),
    #[error("a message from Discord contained invalid field '{0}'")]
    InvalidField(&'static str),
    #[error("an I/O error occured {action}: '{error}'")]
    Io {
        action: &'static str,
        #[source]
        error: std::io::Error,
    },
    #[error("more than 1 URL placeholder used in launch arguments")]
    TooManyUrls,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("encountered unknown variant '{value}' for '{kind}'")]
    UnknownVariant { kind: &'static str, value: u32 },
    #[error(transparent)]
    AppRegistration(#[from] anyhow::Error),
    #[error(transparent)]
    Discord(#[from] DiscordErr),
    #[error("a lobby activity join was not of the form '<lobby_id>:<lobby_secret>'")]
    NonCanonicalLobbyActivitySecret,
    #[error("an asynchronous operation did not complete in the allotted time")]
    TimedOut,
}

impl<T> From<crossbeam_channel::TrySendError<T>> for Error {
    #[inline]
    fn from(se: crossbeam_channel::TrySendError<T>) -> Self {
        match se {
            crossbeam_channel::TrySendError::Full(_) => Self::ChannelFull,
            crossbeam_channel::TrySendError::Disconnected(_) => Self::ChannelDisconnected,
        }
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for Error {
    #[inline]
    fn from(_se: crossbeam_channel::SendError<T>) -> Self {
        Self::ChannelDisconnected
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    #[inline]
    fn from(_se: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelDisconnected
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    #[inline]
    fn from(_se: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::ChannelDisconnected
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    #[inline]
    fn from(_se: tokio::time::error::Elapsed) -> Self {
        Self::TimedOut
    }
}

impl Error {
    #[inline]
    pub(crate) fn io(action: &'static str, error: std::io::Error) -> Self {
        Self::Io { action, error }
    }
}

/// An error related to the actual use of the Discord API.
#[derive(thiserror::Error, Debug)]
pub enum DiscordErr {
    #[error("expected response of '{expected:?}' for request '{nonce}' but received '{actual:?}'")]
    MismatchedResponse {
        expected: crate::CommandKind,
        actual: crate::CommandKind,
        nonce: usize,
    },
    #[error(transparent)]
    Api(#[from] DiscordApiErr),
}

/// An actual API error event sent from Discord. This list is currently incomplete
/// and may change at any time as it is not a documented part of the public API
/// of Discord, eg. the [Game SDK](https://discord.com/developers/docs/game-sdk/discord#data-models)
/// uses a simplified version that collapses a wider range of errors into simpler
/// categories
#[derive(thiserror::Error, Debug)]
pub enum DiscordApiErr {
    #[error("already connected to lobby")]
    AlreadyConnectedToLobby,
    #[error("already connecting to lobby")]
    AlreadyConnectingToLobby,
    #[error("Discord encountered an unknown error processing the command")]
    Unknown,
    #[error("Discord sent an error response with no actual data")]
    NoErrorData,
    #[error("we sent a malformed RPC message to Discord")]
    MalformedCommand,
    #[error("{code:?}: error \"{message:?}\" not specifically known at this time")]
    Generic {
        code: Option<u32>,
        message: Option<String>,
    },
    #[error("secret used to join a lobby was invalid")]
    InvalidLobbySecret,
    #[error("invalid command: {reason}")]
    InvalidCommand { reason: String },
}

impl<'stack> From<Option<crate::types::ErrorPayloadStack<'stack>>> for DiscordApiErr {
    fn from(payload: Option<crate::types::ErrorPayloadStack<'stack>>) -> Self {
        match payload {
            Some(payload) => {
                let code = payload.code;
                let message = payload.message;

                let to_known = |expected: &'static str, err: Self| -> Self {
                    if message.as_deref() == Some(expected) {
                        err
                    } else {
                        Self::Generic {
                            code,
                            message: message.as_ref().map(|s| s.to_string()),
                        }
                    }
                };

                match payload.code {
                    Some(inner) => match inner {
                        1000 => to_known("Unknown Error", Self::Unknown),
                        1003 => to_known("protocol error", Self::MalformedCommand),
                        4000 => Self::InvalidCommand {
                            reason: message
                                .map_or_else(|| "unknown problem".to_owned(), |s| s.into_owned()),
                        },
                        4002 => match message.as_deref() {
                            Some(msg) if msg.starts_with("Invalid command: ") => {
                                Self::InvalidCommand {
                                    reason: msg
                                        .strip_prefix("Invalid command: ")
                                        .unwrap_or("unknown")
                                        .to_owned(),
                                }
                            }
                            _ => Self::Generic {
                                code,
                                message: message.map(|s| s.into_owned()),
                            },
                        },
                        _ => Self::Generic {
                            code,
                            message: message.map(|s| s.into_owned()),
                        },
                    },
                    None => Self::Generic {
                        code,
                        message: message.map(|s| s.into_owned()),
                    },
                }
            }
            None => Self::NoErrorData,
        }
    }
}
