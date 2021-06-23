use crate::{
    lobby::{self, events::LobbyEvent, Lobby, LobbyId},
    user::UserId,
};
use parking_lot::RwLock;

#[derive(Debug)]
pub struct LobbyState {
    pub lobby: Lobby,
    pub messages: Vec<lobby::LobbyMessage>,
}

pub struct LobbyStates {
    pub lobbies: RwLock<Vec<LobbyState>>,
}

impl LobbyStates {
    pub fn new() -> Self {
        Self {
            lobbies: RwLock::new(Vec::new()),
        }
    }

    #[inline]
    fn mut_member(&self, lid: LobbyId, mid: UserId, code: impl FnOnce(&mut lobby::LobbyMember)) {
        let mut lobbies = self.lobbies.write();
        if let Some(l) = lobbies.iter_mut().find(|l| l.lobby.id == lid) {
            if let Some(member) = l.lobby.members.iter_mut().find(|mem| mem.user.id == mid) {
                code(member);
            }
        }
    }

    pub fn on_event(&self, le: LobbyEvent) {
        match le {
            LobbyEvent::Create(lobby) | LobbyEvent::Connect(lobby) => {
                let mut lobbies = self.lobbies.write();
                lobbies.push(LobbyState {
                    lobby,
                    messages: Vec::new(),
                });
            }
            LobbyEvent::Delete { id } => {
                let mut lobbies = self.lobbies.write();
                if let Some(index) = lobbies.iter().position(|l| l.lobby.id == id) {
                    lobbies.swap_remove(index);
                }
            }
            LobbyEvent::Update(lobby) => {
                let mut lobbies = self.lobbies.write();
                if let Some(l) = lobbies.iter_mut().find(|l| l.lobby.id == lobby.id) {
                    l.lobby.capacity = lobby.capacity;
                    l.lobby.kind = lobby.kind;
                    l.lobby.locked = lobby.locked;
                    l.lobby.metadata = lobby.metadata;
                    l.lobby.owner_id = lobby.owner_id;
                }
            }
            LobbyEvent::MemberConnect(me) => {
                let mut lobbies = self.lobbies.write();
                if let Some(l) = lobbies.iter_mut().find(|l| l.lobby.id == me.lobby_id) {
                    l.lobby.members.push(me.member);
                }
            }
            LobbyEvent::MemberDisconnect(me) => {
                let mut lobbies = self.lobbies.write();
                if let Some(l) = lobbies.iter_mut().find(|l| l.lobby.id == me.lobby_id) {
                    if let Some(index) = l
                        .lobby
                        .members
                        .iter()
                        .position(|mem| mem.user.id == me.member.user.id)
                    {
                        l.lobby.members.remove(index);
                    }
                }
            }
            LobbyEvent::MemberUpdate(me) => {
                self.mut_member(me.lobby_id, me.member.user.id, |member| {
                    let speaking = member.speaking;
                    *member = me.member;
                    member.speaking = speaking;
                });
            }
            LobbyEvent::Message(msg) => {
                let mut lobbies = self.lobbies.write();
                if let Some(l) = lobbies.iter_mut().find(|l| l.lobby.id == msg.lobby_id) {
                    l.messages.push(msg.data);
                }
            }
            LobbyEvent::SpeakingStart(se) => {
                self.mut_member(se.lobby_id, se.user_id, |member| {
                    member.speaking = true;
                });
            }
            LobbyEvent::SpeakingStop(se) => {
                self.mut_member(se.lobby_id, se.user_id, |member| {
                    member.speaking = false;
                });
            }
        }
    }
}
