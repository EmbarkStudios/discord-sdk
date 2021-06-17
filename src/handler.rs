use crate::{
    io,
    proto::{self, CommandKind, Event, EventKind, Rpc},
    types::ErrorPayloadStack,
    Error,
};
use crossbeam_channel as cc;

/// Creates a task which receives raw frame buffers and deserializes them, and either
/// notifying the awaiting oneshot for a command response, or in the case of events,
/// broadcasting the event to
pub(crate) fn handler_task(
    handler: Box<dyn crate::DiscordHandler>,
    subscriptions: crate::Subscriptions,
    stx: cc::Sender<Option<Vec<u8>>>,
    mut rrx: tokio::sync::mpsc::Receiver<io::IoMsg>,
    state: crate::State,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        tracing::debug!("starting handler loop");

        let pop_nonce = |nonce: usize| -> Option<crate::NotifyItem> {
            let mut lock = state.notify_queue.lock();

            lock.iter()
                .position(|item| item.nonce == nonce)
                .map(|position| lock.swap_remove(position))
        };

        enum User {
            Event(Event),
            Error(Error),
        }

        // Shunt the user handler to a separate task so that we don't care about it blocking
        // when handling events
        let (user_tx, mut user_rx) = tokio::sync::mpsc::unbounded_channel();
        let user_task = tokio::task::spawn(async move {
            while let Some(event) = user_rx.recv().await {
                match event {
                    User::Event(event) => {
                        handler.on_event(event).await;
                    }
                    User::Error(err) => {
                        handler.on_error(err).await;
                    }
                }
            }
        });

        macro_rules! user_send {
            ($msg:expr) => {
                if user_tx.send($msg).is_err() {
                    tracing::warn!("user handler task has been dropped");
                }
            };
        }

        while let Some(io_msg) = rrx.recv().await {
            let msg = match io_msg {
                io::IoMsg::Disconnected { reason } => {
                    user_send!(User::Event(Event::Disconnected { reason }));
                    continue;
                }
                io::IoMsg::Frame(frame) => process_frame(frame),
            };

            match msg {
                Msg::Event(event) => {
                    if let Event::Ready { .. } = &event {
                        // Spawn a task that subscribes to all of the events
                        // that the caller was interested in when we've finished
                        // the handshake with Discord
                        subscribe_task(subscriptions, stx.clone());
                    }

                    user_send!(User::Event(event));
                }
                Msg::Command { command, kind } => {
                    if kind == CommandKind::Subscribe {
                        tracing::debug!("subscription succeeded: {:#?}", command.inner);
                        continue;
                    }

                    match pop_nonce(command.nonce) {
                        Some(ni) => {
                            if ni
                                .tx
                                .send(if ni.cmd == kind {
                                    Ok(command.inner)
                                } else {
                                    Err(Error::Discord(crate::DiscordErr::MismatchedResponse {
                                        expected: ni.cmd,
                                        actual: kind,
                                        nonce: command.nonce,
                                    }))
                                })
                                .is_err()
                            {
                                tracing::warn!(
                                    cmd = ?kind,
                                    nonce = command.nonce,
                                    "command response dropped as receiver was closed",
                                );
                            }
                        }
                        None => {
                            tracing::warn!(
                                cmd = ?command.inner,
                                nonce = command.nonce,
                                "received a command response with an unknown nonce",
                            );
                        }
                    }
                }
                Msg::Error { nonce, error, .. } => match nonce {
                    Some(nonce) => match pop_nonce(nonce) {
                        Some(ni) => {
                            if let Err(err) = ni.tx.send(Err(error)) {
                                tracing::warn!(
                                    error = ?err,
                                    nonce = nonce,
                                    "error result dropped as receiver was closed",
                                );
                            }
                        }
                        None => {
                            user_send!(User::Error(error));
                        }
                    },
                    None => {
                        user_send!(User::Error(error));
                    }
                },
            }
        }

        drop(user_tx);
        let _ = user_task.await;
    })
}

#[derive(Debug)]
pub(crate) enum Msg {
    Command {
        command: proto::command::CommandFrame,
        kind: CommandKind,
    },
    Event(Event),
    Error {
        nonce: Option<usize>,
        error: Error,
    },
}

fn process_frame(data_buf: Vec<u8>) -> Msg {
    // Discord echoes back our requests with the same nonce they were sent
    // with, however for those echoes, the "evt" field is not set, other than
    // for the "ERROR" RPC type, so we attempt to deserialize those two
    // cases first so we can just ignore the echoes and move on to avoid
    // further complicating the deserialization of the RPCs we actually
    // care about

    #[derive(serde::Deserialize)]
    struct RawMsg {
        cmd: Option<CommandKind>,
        evt: Option<EventKind>,
        #[serde(deserialize_with = "crate::util::string::deserialize_opt")]
        nonce: Option<usize>,
    }

    let rm: RawMsg = match serde_json::from_slice(&data_buf) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(
                "Failed to deserialize message: {} {}",
                e,
                std::str::from_utf8(&data_buf).unwrap(),
            );

            return Msg::Error {
                nonce: None,
                error: Error::Json(e),
            };
        }
    };

    match rm.evt {
        Some(EventKind::Error) => {
            #[derive(serde::Deserialize)]
            struct ErrorMsg<'stack> {
                #[serde(borrow)]
                data: Option<ErrorPayloadStack<'stack>>,
            }

            match serde_json::from_slice::<ErrorMsg<'_>>(&data_buf) {
                Ok(em) => Msg::Error {
                    nonce: rm.nonce,
                    error: Error::Discord(crate::DiscordErr::Api(em.data.into())),
                },
                Err(e) => Msg::Error {
                    nonce: rm.nonce,
                    error: Error::Discord(crate::DiscordErr::Api(crate::DiscordApiErr::Generic {
                        code: None,
                        message: Some(format!("failed to deserialize error: {}", e)),
                    })),
                },
            }
        }
        Some(_) => match serde_json::from_slice::<proto::event::EventFrame>(&data_buf) {
            Ok(event_frame) => Msg::Event(event_frame.inner),
            Err(e) => {
                tracing::warn!(
                    "failed to deserialize event: {:?}",
                    std::str::from_utf8(&data_buf)
                );
                Msg::Error {
                    nonce: rm.nonce,
                    error: Error::Json(e),
                }
            }
        },
        None => match serde_json::from_slice(&data_buf) {
            Ok(cmd_frame) => Msg::Command {
                command: cmd_frame,
                kind: rm
                    .cmd
                    .expect("successfully deserialized command with 'cmd' field"),
            },
            Err(e) => {
                tracing::warn!(
                    "failed to deserialize command: {:?}",
                    std::str::from_utf8(&data_buf)
                );

                Msg::Error {
                    nonce: rm.nonce,
                    error: Error::Json(e),
                }
            }
        },
    }
}

fn subscribe_task(subs: crate::Subscriptions, stx: cc::Sender<Option<Vec<u8>>>) {
    tokio::task::spawn(async move {
        // Assume a max of 64KiB write size and just write all of the
        // subscriptions into a single buffer rather than n
        let mut buffer = Vec::with_capacity(1024);
        let mut nonce = 1usize;

        let mut push = |kind: EventKind| {
            #[cfg(target_pointer_width = "32")]
            let nunce = 0x10000000 | nonce;
            #[cfg(target_pointer_width = "64")]
            let nunce = 0x1000000000000000 | nonce;

            let _ = io::serialize_message(
                io::OpCode::Frame,
                &Rpc::<()> {
                    cmd: CommandKind::Subscribe,
                    evt: Some(kind),
                    nonce: nunce.to_string(),
                    args: None,
                },
                &mut buffer,
            );

            nonce += 1;
        };

        let activity = if subs.contains(crate::Subscriptions::ACTIVITY) {
            [
                EventKind::ActivityInvite,
                EventKind::ActivityJoin,
                EventKind::ActivityJoinRequest,
                EventKind::ActivitySpectate,
            ]
            .iter()
        } else {
            [].iter()
        };

        let lobby = if subs.contains(crate::Subscriptions::LOBBY) {
            [
                EventKind::LobbyDelete,
                EventKind::LobbyMemberConnect,
                EventKind::LobbyMemberDisconnect,
                EventKind::LobbyMemberUpdate,
                EventKind::LobbyMessage,
                EventKind::LobbyUpdate,
                EventKind::SpeakingStart,
                EventKind::SpeakingStop,
            ]
            .iter()
        } else {
            [].iter()
        };

        let user = if subs.contains(crate::Subscriptions::USER) {
            [EventKind::CurrentUserUpdate].iter()
        } else {
            [].iter()
        };

        activity.chain(lobby).chain(user).for_each(|kind| {
            push(*kind);
        });

        // Unlike EVERY other event, subscribing to OVERLAY_UPDATE requires
        // an argument... :facepalm:
        if subs.contains(crate::Subscriptions::OVERLAY) {
            #[cfg(target_pointer_width = "32")]
            let nunce = 0x10000000 | nonce;
            #[cfg(target_pointer_width = "64")]
            let nunce = 0x1000000000000000 | nonce;

            let _ = io::serialize_message(
                io::OpCode::Frame,
                &Rpc {
                    cmd: CommandKind::Subscribe,
                    evt: Some(EventKind::OverlayUpdate),
                    nonce: nunce.to_string(),
                    args: Some(crate::overlay::OverlayPidArgs::new()),
                },
                &mut buffer,
            );

            //nonce += 1;
        }

        if stx.send(Some(buffer)).is_err() {
            tracing::warn!("unable to send subscription RPCs to I/O task");
        }
    });
}
