#![allow(unused_must_use, clippy::dbg_macro)]

use dgs::Discord;
use discord_game_sdk as dgs;
use structopt::StructOpt;

#[derive(StructOpt)]
enum LobbyCmd {
    Create {
        #[structopt(long)]
        capacity: Option<u32>,
    },
    Update {
        #[structopt(long, default_value = "4")]
        capacity: u32,
    },
    Delete,
    Disconnect,
    Sequence,
}

#[derive(StructOpt)]
enum InputMode {
    Activity,
    Ptt,
}

#[derive(StructOpt)]
enum VoiceCmd {
    GetInputMode,
    SetInputMode(InputMode),
    GetSelfMute,
    SetSelfMute {
        #[structopt(long)]
        mute: bool,
    },
    GetSelfDeaf,
    SetSelfDeaf {
        #[structopt(long)]
        deaf: bool,
    },
    GetLocalMute {
        user: i64,
    },
    SetLocalMute {
        #[structopt(long)]
        mute: bool,
        user: i64,
    },
    GetLocalVolume {
        user: i64,
    },
    SetLocalVolume {
        #[structopt(long, default_value = "100")]
        val: u8,
        user: i64,
    },
}

#[derive(StructOpt)]
enum ActivityCmd {
    Invite { id: String },
    Accept,
    UpdatePresence,
}

#[derive(StructOpt)]
enum OverlayCmd {
    Enabled,
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

#[derive(StructOpt)]
enum RelationCmd {
    List,
}

#[derive(StructOpt)]
enum Cmd {
    Lobby(LobbyCmd),
    Activity(ActivityCmd),
    Overlay(OverlayCmd),
    Relations(RelationCmd),
    Voice(VoiceCmd),
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
    let current_lobby = std::sync::Arc::new(std::sync::Mutex::new(None));

    let mut update_count = 0;

    loop {
        discord.run_callbacks().unwrap();
        line.clear();

        if std::io::stdin().read_line(&mut line).is_ok() {
            line.pop();

            if line.is_empty() {
                continue;
            }

            match Cmd::from_iter_safe(std::iter::once("discord").chain(line.split(' '))) {
                Ok(cmd) => match cmd {
                    Cmd::Relations(rc) => {
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
                    Cmd::Overlay(overlay) => match overlay {
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
                    Cmd::Voice(voice) => match voice {
                        VoiceCmd::GetInputMode => {
                            dbg!(discord.input_mode());
                        }
                        VoiceCmd::SetInputMode(im) => {
                            let ttx = tx.clone();

                            let im = match im {
                                InputMode::Activity => dgs::InputMode::voice_activity(),
                                InputMode::Ptt => dgs::InputMode::push_to_talk("ctrl + a"),
                            };

                            discord.set_input_mode(im, move |_, res| {
                                dbg!(res);
                                ttx.send(()).unwrap();
                            });

                            wait!();
                        }
                        VoiceCmd::GetSelfMute => {
                            dbg!(discord.self_muted());
                        }
                        VoiceCmd::SetSelfMute { mute } => {
                            dbg!(discord.set_self_mute(mute));
                        }
                        VoiceCmd::GetSelfDeaf => {
                            dbg!(discord.self_deafened());
                        }
                        VoiceCmd::SetSelfDeaf { deaf } => {
                            dbg!(discord.set_self_deaf(deaf));
                        }
                        VoiceCmd::GetLocalMute { user } => {
                            dbg!(discord.local_muted(user));
                        }
                        VoiceCmd::SetLocalMute { mute, user } => {
                            dbg!(discord.set_local_mute(user, mute));
                        }
                        VoiceCmd::GetLocalVolume { user } => {
                            dbg!(discord.local_volume(user));
                        }
                        VoiceCmd::SetLocalVolume { val, user } => {
                            dbg!(discord.set_local_volume(user, val));
                        }
                    },
                    Cmd::Activity(activity) => match activity {
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
                    Cmd::Lobby(lobby) => match lobby {
                        LobbyCmd::Create { capacity } => {
                            {
                                let mut lock = current_lobby.lock().unwrap();

                                if let Some(cl) = &*lock {
                                    let current_lobby = current_lobby.clone();
                                    discord.delete_lobby(*cl, move |_discord, res| match res {
                                        Ok(_) => {
                                            println!(
                                                "DELETED LOBBY {:?}",
                                                current_lobby.lock().unwrap()
                                            );
                                        }
                                        Err(e) => eprintln!("FAILED TO DELETE LOBBY: {}", e),
                                    });

                                    *lock = None;
                                }
                            }

                            let mut lobby_transaction = dgs::LobbyTransaction::new();
                            if let Some(capacity) = capacity {
                                lobby_transaction.capacity(capacity);
                            }
                            //lobby_transaction.kind(dgs::LobbyKind::Private);
                            let current_lobby = current_lobby.clone();
                            discord.create_lobby(&lobby_transaction, move |_discord, lobby| {
                                match lobby {
                                    Ok(lobby) => {
                                        *current_lobby.lock().unwrap() = Some(lobby.id());
                                        println!("LOBBY CREATED: {:#?}", lobby);
                                    }
                                    Err(e) => eprintln!("FAILED TO CREATE LOBBY: {}", e),
                                }
                            });
                        }
                        LobbyCmd::Update { capacity } => {
                            let lobby_id = {
                                let lock = current_lobby.lock().unwrap();

                                match &*lock {
                                    Some(id) => *id,
                                    None => {
                                        eprintln!("LOBBY NOT CREATED");
                                        continue;
                                    }
                                }
                            };

                            let mut lobby_transaction = dgs::LobbyTransaction::new();
                            lobby_transaction.capacity(capacity);

                            if update_count % 2 == 0 {
                                lobby_transaction.add_metadata("first".to_owned(), "1".to_owned());
                                lobby_transaction.add_metadata("second".to_owned(), "2".to_owned());
                            } else {
                                lobby_transaction.delete_metadata::<String>("first".to_owned());
                                lobby_transaction.delete_metadata::<String>("second".to_owned());
                            }

                            update_count += 1;

                            discord.update_lobby(
                                lobby_id,
                                &lobby_transaction,
                                move |_discord, lobby| match lobby {
                                    Ok(lobby) => {
                                        println!("LOBBY UPDATED: {:#?}", lobby);
                                    }
                                    Err(e) => eprintln!("FAILED TO UPDATE LOBBY: {}", e),
                                },
                            );
                        }
                        LobbyCmd::Delete => {
                            let mut lock = current_lobby.lock().unwrap();

                            if let Some(cl) = &*lock {
                                let current_lobby = current_lobby.clone();

                                discord.delete_lobby(*cl, move |_discord, res| match res {
                                    Ok(_) => {
                                        println!(
                                            "DELETED LOBBY {:?}",
                                            current_lobby.lock().unwrap()
                                        );
                                    }
                                    Err(e) => eprintln!("FAILED TO DELETE LOBBY: {}", e),
                                });

                                *lock = None;
                            }
                        }
                        LobbyCmd::Disconnect => {
                            let lock = current_lobby.lock().unwrap();

                            if let Some(cl) = &*lock {
                                let current_lobby = current_lobby.clone();
                                discord.disconnect_lobby(*cl, move |_discord, res| match res {
                                    Ok(_) => println!(
                                        "DISCONNECTED FROM LOBBY {:?}",
                                        current_lobby.lock().unwrap()
                                    ),
                                    Err(e) => eprintln!("FAILED TO DISCONNECT FROM LOBBY: {}", e),
                                });
                            }
                        }
                        LobbyCmd::Sequence => {
                            let mut lobby_transaction = dgs::LobbyTransaction::new();
                            lobby_transaction.capacity(5);
                            lobby_transaction.kind(dgs::LobbyKind::Public);
                            lobby_transaction.locked(false);
                            lobby_transaction.add_metadata("one".to_owned(), "1".to_owned());

                            let lobby_secret = std::sync::Arc::new(std::sync::Mutex::new(None));

                            let cl = current_lobby.clone();
                            let ls = lobby_secret;
                            let ttx = tx.clone();
                            discord.create_lobby(&lobby_transaction, move |dis, res| {
                                match res {
                                    Ok(lobby) => {
                                        *cl.lock().unwrap() = Some(lobby.id());

                                        dbg!(lobby.secret());
                                        *ls.lock().unwrap() = Some(dbg!(dis
                                            .lobby_activity_secret(lobby.id())
                                            .unwrap()));

                                        println!("CREATED LOBBY: {:#?}", lobby);
                                    }
                                    Err(e) => {
                                        panic!("FAILED TO CREATE LOBBY: {}", e);
                                    }
                                }

                                ttx.send(()).unwrap();
                            });

                            wait!();

                            let lobby_id =
                                current_lobby.lock().unwrap().expect("expected lobby id");

                            let mut lobby_tx = dgs::LobbyTransaction::new();
                            lobby_tx.delete_metadata::<String>("one".to_owned());
                            lobby_tx.add_metadata("two".to_owned(), "2".to_owned());
                            lobby_tx.kind(dgs::LobbyKind::Private);

                            let ttx = tx.clone();
                            discord.update_lobby(lobby_id, &lobby_tx, move |_, res| {
                                match res {
                                    Ok(lobby) => {
                                        println!("UPDATED LOBBY: {:#?}", lobby);
                                    }
                                    Err(e) => {
                                        panic!("FAILED TO UPDATE LOBBY: {}", e);
                                    }
                                }

                                ttx.send(()).unwrap();
                            });

                            //activity!("updated, searching");

                            wait!();

                            let mut search = dgs::SearchQuery::new();
                            search.distance(dgs::Distance::Global);
                            // search.filter(
                            //     "metadata.two".to_owned(),
                            //     dgs::Comparison::Equal,
                            //     "2".to_owned(),
                            //     dgs::Cast::Number,
                            // );
                            // search.sort(
                            //     "owner_id".to_owned(),
                            //     discord.current_user().unwrap().id().to_string(),
                            //     dgs::Cast::Number,
                            // );

                            let ttx = tx.clone();
                            discord.lobby_search(&search, move |discord, result| {
                                result.expect("failed to search lobbies");

                                let count = dbg!(discord.lobby_count());

                                for i in 0..count {
                                    if let Ok(id) = discord.lobby_id_at(i) {
                                        dbg!(discord.lobby(id)).unwrap();
                                    }
                                }

                                ttx.send(()).unwrap();
                            });

                            wait!();

                            //activity!("searched, disconnecting");
                            // let ttx = tx.clone();
                            // discord.disconnect_lobby(lobby_id, move |_, res| {
                            //     res.expect("failed to disconnect from lobby");
                            //     ttx.send(());
                            // });

                            // wait!();

                            // //activity!("disconnected, connecting");
                            // let ttx = tx.clone();
                            // discord.connect_lobby_with_activity_secret(
                            //     lobby_secret
                            //         .lock()
                            //         .unwrap()
                            //         .deref()
                            //         .as_ref()
                            //         .map(|s| s.clone())
                            //         .unwrap(),
                            //     move |_, lobby| {
                            //         match lobby {
                            //             Ok(lobby) => {
                            //                 println!("CONNECTED LOBBY: {:#?}", lobby);
                            //             }
                            //             Err(e) => {
                            //                 eprintln!("FAILED TO CONNECT TO LOBBY: {}", e);
                            //             }
                            //         }

                            //         ttx.send(());
                            //     },
                            // );

                            // wait!();

                            let ttx = tx.clone();
                            discord.send_lobby_message(
                                lobby_id,
                                "a very good message",
                                move |_, res| {
                                    dbg!(res).unwrap();
                                    ttx.send(()).unwrap();
                                },
                            );

                            wait!();

                            let ttx = tx.clone();
                            discord.connect_lobby_voice(lobby_id, move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();

                            let ttx = tx.clone();
                            discord.disconnect_lobby_voice(lobby_id, move |_, res| {
                                dbg!(res).unwrap();
                                ttx.send(()).unwrap();
                            });

                            wait!();

                            let user_id = discord.current_user().unwrap().id();
                            let mut member_tx = dgs::LobbyMemberTransaction::new();
                            member_tx.add_metadata("three".to_owned(), "3".to_owned());
                            member_tx.add_metadata("four".to_owned(), "4".to_owned());

                            let ttx = tx.clone();
                            discord.update_member(lobby_id, user_id, &member_tx, move |_, res| {
                                dbg!(res).unwrap();

                                ttx.send(()).unwrap();
                            });

                            wait!();

                            //activity!("connected, deleting");
                            let ttx = tx;
                            discord.delete_lobby(lobby_id, move |_, res| {
                                res.expect("failed to delete lobby");

                                ttx.send(()).unwrap();
                            });

                            wait!();

                            break;
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
