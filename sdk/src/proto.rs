pub(crate) mod command;
pub(crate) mod event;

pub(crate) use command::{Command, CommandKind};
pub(crate) use event::{Event, EventKind};

#[derive(serde::Serialize)]
pub(crate) struct Rpc<T> {
    /// The RPC type
    pub(crate) cmd: CommandKind,
    /// Every RPC we send to Discord needs a [`nonce`](https://en.wikipedia.org/wiki/Cryptographic_nonce)
    /// to uniquely identify the RPC. This nonce is sent back when Discord either
    /// responds to an RPC, or acknowledges receipt
    pub(crate) nonce: String,
    /// The event, only used for un/subscribe RPCs :(
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) evt: Option<EventKind>,
    /// The arguments for the RPC, used by all RPCs other than un/subscribe :(
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) args: Option<T>,
}
