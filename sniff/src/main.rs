#![allow(unused_must_use, clippy::dbg_macro)]

use clap::{Parser, Subcommand};
use dgs::Discord;
use discord_game_sdk as dgs;

#[derive(Subcommand)]
enum ActivityCmd {
    Invite { id: String },
    Accept,
    UpdatePresence,
}

#[derive(Subcommand)]
enum OverlayCmd {
    Enabled,
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

#[derive(Subcommand)]
enum RelationCmd {
    List,
}

#[derive(Subcommand)]
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
        cmd: RelationCmd,
    },
}

#[derive(Parser)]
struct Cmd {
    #[clap(subcommand)]
    cmd: Commands,
}

fn main() {
    let mut discord = Discord::new(310270644849737729).unwrap();
    *discord.event_handler_mut() = Some(Printer);

    let (tx, rx) = std::sync::mpsc::channel();

    macro_rules! wait {
        () => {
            loop {
                discord.run_callbacks().unwrap();
                if rx.try_recv().is_ok() {
                    break;
                }
            }
        };
    }

    let mut line = String::new();

    loop {
        discord.run_callbacks().unwrap();
        line.clear();

        if std::io::stdin().read_line(&mut line).is_ok() {
            line.pop();

            if line.is_empty() {
                continue;
            }

            match Cmd::try_parse_from(std::iter::once("discord").chain(line.split(' '))) {
                Ok(cmd) => match cmd.cmd {
                    Commands::Relations { cmd: rc } => {
                        match rc {
                            RelationCmd::List => {
                                //let ttx = tx.clone();
                                match discord.iter_relationships() {
                                    Err(e) => eprintln!("{:#}", e),
                                    Ok(iter) => {
                                        for (i, rel) in iter.enumerate() {
                                            println!("{} {:#?}", i, rel);
                                        }
                                    }
                                }

                                //wait!();
                            }
                        }
                    }
                    Commands::Overlay { cmd: overlay } => match overlay {
                        OverlayCmd::Enabled => {
                            //let ttx = tx.clone();
                            dbg!(discord.overlay_enabled());

                            //wait!();
                        }
                        OverlayCmd::Open => {
                            let ttx = tx.clone();
                            discord.set_overlay_opened(true, move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                        OverlayCmd::Close => {
                            let ttx = tx.clone();
                            discord.set_overlay_opened(false, move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                        OverlayCmd::Invite { join } => {
                            let ttx = tx.clone();
                            discord.open_invite_overlay(
                                if join {
                                    dgs::Action::Join
                                } else {
                                    dgs::Action::Spectate
                                },
                                move |_, res| {
                                    dbg!(res).unwrap();
                                    ttx.send(()).unwrap();
                                },
                            );

                            wait!();
                        }
                        OverlayCmd::Voice => {
                            let ttx = tx.clone();
                            discord.open_voice_settings(move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                        OverlayCmd::GuildInvite { code } => {
                            let ttx = tx.clone();
                            discord.open_guild_invite_overlay(code, move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                    },
                    Commands::Activity { cmd: activity } => match activity {
                        ActivityCmd::UpdatePresence => {
                            let mut activity = dgs::Activity::empty();

                            activity
                                .with_state("state")
                                .with_details("details")
                                .with_party_amount(1)
                                .with_party_capacity(2)
                                .with_party_id("muchuniqueveryid")
                                .with_join_secret("muchsecretveryjoin")
                                .with_instance(true);

                            let ttx = tx.clone();
                            discord.update_activity(&activity, move |_dis, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                        ActivityCmd::Invite { id } => {
                            let user_id = id.parse().unwrap();

                            let mut activity = dgs::Activity::empty();

                            activity
                                .with_state("state")
                                .with_details("details")
                                .with_party_amount(1)
                                .with_party_capacity(2)
                                .with_party_id("muchuniqueveryid")
                                .with_join_secret("muchsecretveryjoin")
                                .with_instance(true);

                            let ttx = tx.clone();
                            discord.update_activity(&activity, move |dis, res| {
                                dbg!(res).unwrap();

                                dis.send_invite(
                                    user_id,
                                    dgs::Action::Join,
                                    "joinsie woinsie my funsie onesie",
                                    move |_dis, res| {
                                        dbg!(res).unwrap();

                                        ttx.send(()).unwrap();
                                    },
                                );
                            });

                            wait!();
                        }
                        ActivityCmd::Accept => {
                            let ttx = tx.clone();
                            discord.accept_invite(682969165652689005, move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                    },
                },
                Err(e) => {
                    eprintln!("Failed to parse - {}", e);
                }
            }
        }
    }
}

struct Printer;

impl dgs::EventHandler for Printer {
    fn on_user_achievement_update(
        &mut self,
        _discord: &Discord<'_, Self>,
        user_achievement: &dgs::UserAchievement,
    ) {
        println!("USER ACHIEVEMENT UPDATE: {:#?}", user_achievement);
    }

    fn on_activity_join(&mut self, _discord: &Discord<'_, Self>, secret: &str) {
        println!("ACTIVITY JOIN: {}", secret);
    }

    fn on_activity_spectate(&mut self, _discord: &Discord<'_, Self>, secret: &str) {
        println!("ACTIVITY SPECTATE: {}", secret);
    }

    fn on_activity_join_request(&mut self, discord: &Discord<'_, Self>, user: &dgs::User) {
        println!("ACTIVITY JOIN REQUEST: {:#?}", user);

        discord.send_request_reply(user.id(), dgs::RequestReply::No, |_, res| {
            println!("ACTIVITY JOIN REQUEST REPLY: {:#?}", res);
        });
    }

    fn on_activity_invite(
        &mut self,
        _discord: &Discord<'_, Self>,
        kind: dgs::Action,
        user: &dgs::User,
        activity: &dgs::Activity,
    ) {
        println!(
            "ACTIVITY INVITE: kind = {:?} user = {:#?} activity = {:#?}",
            kind, user, activity
        );
    }

    fn on_lobby_update(&mut self, _discord: &Discord<'_, Self>, lobby_id: dgs::LobbyID) {
        println!("LOBBY UPDATE: {}", lobby_id);
    }

    fn on_lobby_delete(
        &mut self,
        _discord: &Discord<'_, Self>,
        lobby_id: dgs::LobbyID,
        reason: u32,
    ) {
        println!("LOBBY DELETED: {} - {}", lobby_id, reason);
    }

    fn on_member_connect(
        &mut self,
        _discord: &Discord<'_, Self>,
        lobby_id: dgs::LobbyID,
        member_id: dgs::UserID,
    ) {
        println!("MEMBER CONNECTED: {} - {}", lobby_id, member_id);
    }

    fn on_member_update(
        &mut self,
        _discord: &Discord<'_, Self>,
        lobby_id: dgs::LobbyID,
        member_id: dgs::UserID,
    ) {
        println!("MEMBER UPDATED: {} - {}", lobby_id, member_id);
    }

    fn on_member_disconnect(
        &mut self,
        _discord: &Discord<'_, Self>,
        lobby_id: dgs::LobbyID,
        member_id: dgs::UserID,
    ) {
        println!("MEMBER DISCONNECTED: {} - {}", lobby_id, member_id);
    }

    fn on_lobby_message(
        &mut self,
        _discord: &Discord<'_, Self>,
        lobby_id: dgs::LobbyID,
        member_id: dgs::UserID,
        data: &[u8],
    ) {
        println!(
            "LOBBY MESSAGE: {} - {} - {:?}",
            lobby_id,
            member_id,
            std::str::from_utf8(data)
        );
    }

    fn on_current_user_update(&mut self, _discord: &Discord<'_, Self>) {
        println!("USER UPDATED",);
    }

    fn on_relationship_update(
        &mut self,
        _discord: &Discord<'_, Self>,
        relationship: &dgs::Relationship,
    ) {
        println!("RELATIONSHIP UPDATE: {:#?}", relationship);
    }

    fn on_relationships_refresh(&mut self, _discord: &Discord<'_, Self>) {
        println!("RELATIONSHIP REFRESHED");
    }
}
