use examples_shared::{
    self as es,
    anyhow::{self, Context as _},
    ds, tokio, tracing,
};
use structopt::StructOpt;

use ds::{activity, lobby, overlay, relations};

#[derive(StructOpt, Debug)]
enum LobbyCmd {
    Create {
        #[structopt(long, default_value = "4")]
        capacity: u32,
    },
    Update {
        #[structopt(long, default_value = "4")]
        capacity: u32,
    },
    Delete,
    Connect {
        #[structopt(long)]
        id: String,
        #[structopt(long)]
        secret: String,
    },
    Disconnect {
        #[structopt(long)]
        id: String,
    },
    Msg {
        #[structopt(long)]
        id: String,
        msg: String,
    },
    Print,
    Search,
}

#[derive(StructOpt, Debug)]
struct ActivityUpdateCmd {
    #[structopt(long, default_value = "")]
    state: String,
    #[structopt(long, default_value = "")]
    details: String,
    /// Sets the start timestamp to the current system time
    #[structopt(long)]
    start: bool,
    /// Sets the end timestamp to 1 minute in the future
    #[structopt(long)]
    end: bool,
}

#[derive(StructOpt, Debug)]
enum ActivityCmd {
    Invite {
        /// The message to send to the user in the invite
        #[structopt(long, default_value = "please join")]
        msg: String,
        /// Invite to spectate, if not provided, invites to join instead
        #[structopt(long)]
        spectate: bool,
        /// The unique identifier for the user
        id: String,
    },
    Accept,
    Reply {
        #[structopt(long)]
        accept: bool,
    },
    Update(ActivityUpdateCmd),
}

#[derive(StructOpt, Debug)]
enum OverlayCmd {
    Open,
    Close,
    Invite {
        #[structopt(long)]
        join: bool,
    },
    Voice,
    GuildInvite {
        code: String,
    },
}

#[derive(StructOpt, Debug)]
enum RelationsCmd {
    Print,
}

#[derive(StructOpt, Debug)]
enum Cmd {
    Lobby(LobbyCmd),
    Activity(ActivityCmd),
    Overlay(OverlayCmd),
    Relations(RelationsCmd),
    Exit,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = es::make_client(ds::Subscriptions::ALL).await;

    //let user = client.user;
    let wheel = client.wheel;
    let discord = client.discord;

    let (invites_tx, invites_rx) = ds::cc::unbounded();
    let (joins_tx, joins_rx) = ds::cc::unbounded();

    let mut activity_events = wheel.activity().0;
    tokio::task::spawn(async move {
        use activity::events::ActivityEvent;
        while let Ok(ae) = activity_events.recv().await {
            match ae {
                ActivityEvent::Invite(invite) => {
                    if invites_tx.send(invite).is_err() {
                        break;
                    }
                }
                ActivityEvent::JoinRequest(jre) => {
                    tracing::info!("Received join request from {}", jre.user);
                    if joins_tx.send(jre.user.id).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    let mut lobby_events = wheel.lobby().0;
    let lobby_states = std::sync::Arc::new(lobby::state::LobbyStates::new());
    let ls = lobby_states.clone();
    tokio::task::spawn(async move {
        while let Ok(le) = lobby_events.recv().await {
            tracing::info!(event = ?le, "lobby event");
            ls.on_event(le);
        }
    });

    let relationships = discord.get_relationships().await?;

    let mut rl_events = wheel.relationships().0;

    let relationships = std::sync::Arc::new(relations::state::Relationships::new(relationships));
    let rs = relationships.clone();
    tokio::task::spawn(async move {
        while let Ok(re) = rl_events.recv().await {
            tracing::info!(event = ?re, "relationship event");
            rs.on_event(re);
        }
    });

    struct ReplState {
        invites_rx: ds::cc::Receiver<activity::events::InviteEvent>,
        joins_rx: ds::cc::Receiver<ds::user::UserId>,
        created_lobby: Option<lobby::Lobby>,
        lobbies: std::sync::Arc<lobby::state::LobbyStates>,
        relationships: std::sync::Arc<relations::state::Relationships>,
    }

    let mut repl_state = ReplState {
        invites_rx,
        joins_rx,
        created_lobby: None,
        lobbies: lobby_states,
        relationships,
    };

    let mut line = String::new();
    loop {
        line.clear();

        let _ = std::io::stdin().read_line(&mut line);
        // Remove trailing newline
        line.pop();

        let split = match shell_words::split(&line) {
            Ok(sl) => sl,
            Err(e) => {
                tracing::error!("failed to split command: {}", e);
                continue;
            }
        };

        match Cmd::from_iter_safe(std::iter::once("repl".to_owned()).chain(split.into_iter())) {
            Ok(cmd) => {
                if let Cmd::Exit = &cmd {
                    break;
                }

                async fn process(
                    discord: &ds::Discord,
                    cmd: &Cmd,
                    state: &mut ReplState,
                ) -> anyhow::Result<()> {
                    match cmd {
                        Cmd::Exit => unreachable!(),
                        Cmd::Lobby(lobby) => match lobby {
                            LobbyCmd::Create { capacity } => {
                                if let Some(lobby) = &state.created_lobby {
                                    anyhow::bail!("Lobby {:#?} already exists", lobby);
                                }

                                let lobby = discord
                                    .create_lobby(
                                        ds::lobby::CreateLobbyBuilder::new()
                                            .capacity(std::num::NonZeroU32::new(*capacity)),
                                    )
                                    .await?;

                                tracing::info!(lobby = ?lobby, "created");

                                discord.connect_lobby_voice(lobby.id).await?;
                                state.created_lobby = Some(lobby);
                            }
                            LobbyCmd::Delete => match &mut state.created_lobby {
                                Some(lobby) => {
                                    discord.delete_lobby(lobby.id).await?;
                                    state.created_lobby = None;
                                }
                                None => {
                                    anyhow::bail!("No lobby to delete");
                                }
                            },
                            LobbyCmd::Connect { id, secret } => {
                                let id: lobby::LobbyId = id.parse().context("invalid lobby id")?;

                                discord
                                    .connect_lobby(ds::lobby::ConnectLobby {
                                        id,
                                        secret: secret.clone(),
                                    })
                                    .await?;

                                discord.connect_lobby_voice(id).await?;
                            }
                            LobbyCmd::Update { capacity } => match &mut state.created_lobby {
                                Some(lobby) => {
                                    let args = discord
                                        .update_lobby(
                                            lobby::UpdateLobbyBuilder::new(lobby)
                                                .capacity(std::num::NonZeroU32::new(*capacity)),
                                        )
                                        .await?;

                                    args.modify(lobby);
                                }
                                None => {
                                    anyhow::bail!("No lobby to update");
                                }
                            },
                            LobbyCmd::Disconnect { id } => {
                                let id: lobby::LobbyId = id.parse().context("invalid lobby id")?;

                                discord.disconnect_lobby(id).await?;
                            }
                            LobbyCmd::Msg { id, msg } => {
                                let id: lobby::LobbyId = id.parse().context("invalid lobby id")?;

                                discord
                                    .send_lobby_message(id, lobby::LobbyMessage::Text(msg.clone()))
                                    .await?;
                            }
                            LobbyCmd::Search => {
                                let query = ds::lobby::search::SearchQuery::default();
                                let lobbies = discord.search_lobbies(query).await?;

                                tracing::info!("found lobbies: {:#?}", lobbies);
                            }
                            LobbyCmd::Print => {
                                tracing::info!("{:#?}", state.lobbies.lobbies.read());
                            }
                        },
                        Cmd::Activity(activity) => match activity {
                            ActivityCmd::Accept => {
                                let invite =
                                    state.invites_rx.try_recv().context("no pending invites")?;

                                discord.accept_invite(&invite).await?;
                            }
                            ActivityCmd::Reply { accept } => {
                                let user = state
                                    .joins_rx
                                    .try_recv()
                                    .context("no pending join requests")?;

                                discord.send_join_request_reply(user, *accept).await?;
                            }
                            ActivityCmd::Invite { id, msg, spectate } => {
                                let user_id = id.parse().context("invalid user id")?;
                                discord
                                    .invite_user(
                                        user_id,
                                        msg,
                                        if *spectate {
                                            activity::ActivityActionKind::Spectate
                                        } else {
                                            activity::ActivityActionKind::Join
                                        },
                                    )
                                    .await?;
                            }
                            ActivityCmd::Update(update) => {
                                let ab = activity::ActivityBuilder::new()
                                    .state(&update.state)
                                    .details(&update.details)
                                    .party(
                                        format!("repl-{}", std::process::id()),
                                        std::num::NonZeroU32::new(1),
                                        std::num::NonZeroU32::new(2),
                                        activity::PartyPrivacy::Private,
                                    )
                                    .secrets(ds::activity::Secrets {
                                        join: Some("joinme".to_owned()),
                                        spectate: Some("spectateme".to_owned()),
                                        r#match: None,
                                    })
                                    .timestamps(
                                        update.start.then(std::time::SystemTime::now),
                                        update.end.then(|| {
                                            std::time::SystemTime::now()
                                                + std::time::Duration::from_secs(60)
                                        }),
                                    );

                                discord.update_activity(ab).await?;
                            }
                        },
                        Cmd::Overlay(overlay) => match overlay {
                            OverlayCmd::Open => {
                                discord
                                    .set_overlay_visibility(overlay::Visibility::Visible)
                                    .await?;
                            }
                            OverlayCmd::Close => {
                                discord
                                    .set_overlay_visibility(overlay::Visibility::Hidden)
                                    .await?;
                            }
                            OverlayCmd::Invite { join } => {
                                discord
                                    .open_activity_invite(if *join {
                                        overlay::InviteAction::Join
                                    } else {
                                        overlay::InviteAction::Spectate
                                    })
                                    .await?;
                            }
                            OverlayCmd::Voice => {
                                tracing::warn!(
                                    "Not sending the overlay voice settings RPC as Discord will crash"
                                );
                                //client.open_voice_settings().await?;
                            }
                            OverlayCmd::GuildInvite { code } => {
                                discord.open_guild_invite(code).await?;
                            }
                        },
                        Cmd::Relations(rc) => match rc {
                            RelationsCmd::Print => {
                                tracing::info!("{:#?}", state.relationships.relationships.read());
                            }
                        },
                    }

                    Ok(())
                }

                if let Err(e) = process(&discord, &cmd, &mut repl_state).await {
                    tracing::error!("{:#?} failed - {:#}", cmd, e);
                }
            }
            Err(e) => {
                tracing::error!("{}", e);
                continue;
            }
        }
    }

    discord.disconnect().await;

    Ok(())
}
