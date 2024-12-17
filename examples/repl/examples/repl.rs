use clap::{Parser, Subcommand};
use examples_shared::{
    self as es,
    anyhow::{self, Context as _},
    ds, tokio, tracing,
};

use ds::{activity, overlay, relations};

#[derive(Parser, Debug)]
struct ActivityUpdateCmd {
    #[clap(long, default_value = "repling")]
    state: String,
    #[clap(long, default_value = "having fun")]
    details: String,
    /// Sets the start timestamp to the current system time
    #[clap(long)]
    start: bool,
    /// Sets the end timestamp to 1 minute in the future
    #[clap(long)]
    end: bool,
}

#[derive(Subcommand, Debug)]
enum ActivityCmd {
    Invite {
        /// The message to send to the user in the invite
        #[clap(long, default_value = "please join")]
        msg: String,
        /// Invite to spectate, if not provided, invites to join instead
        #[clap(long)]
        spectate: bool,
        /// The unique identifier for the user
        id: String,
    },
    Accept,
    Reply {
        #[clap(long)]
        accept: bool,
    },
    Update(ActivityUpdateCmd),
}

#[derive(Subcommand, Debug)]
enum OverlayCmd {
    Open,
    Close,
    Invite {
        #[clap(long)]
        join: bool,
    },
    Voice,
    GuildInvite {
        code: String,
    },
}

#[derive(Subcommand, Debug)]
enum RelationsCmd {
    Print,
}

#[derive(Clone, clap::Args, Debug)]
struct InputMode {
    #[clap(conflicts_with = "ptt")]
    activity: bool,
    ptt: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Activity {
        #[clap(subcommand)]
        cmd: ActivityCmd,
    },
    Overlay {
        #[clap(subcommand)]
        cmd: OverlayCmd,
    },
    Relations {
        #[clap(subcommand)]
        cmd: RelationsCmd,
    },
    Exit,
}

#[derive(Parser, Debug)]
struct Cmd {
    #[clap(subcommand)]
    cmd: Commands,
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
        relationships: std::sync::Arc<relations::state::Relationships>,
    }

    let mut repl_state = ReplState {
        invites_rx,
        joins_rx,
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

        match Cmd::try_parse_from(std::iter::once("repl".to_owned()).chain(split)) {
            Ok(cmd) => {
                if let Commands::Exit = &cmd.cmd {
                    break;
                }

                async fn process(
                    discord: &ds::Discord,
                    cmd: &Cmd,
                    state: &mut ReplState,
                ) -> anyhow::Result<()> {
                    match &cmd.cmd {
                        Commands::Exit => unreachable!(),
                        Commands::Activity { cmd: activity } => match activity {
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
                        Commands::Overlay { cmd: overlay } => match overlay {
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
                        Commands::Relations { cmd: rc } => match rc {
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
