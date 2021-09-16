use crate::voice::{self, events::VoiceEvent};
use parking_lot::RwLock;

pub use voice::events::VoiceSettingsUpdateEvent as VoiceStateInner;

pub struct VoiceState {
    pub state: RwLock<VoiceStateInner>,
}

impl VoiceState {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(VoiceStateInner::default()),
        }
    }

    pub fn on_event(&self, ve: VoiceEvent) {
        match ve {
            VoiceEvent::Refresh(refresh) => {
                *self.state.write() = refresh;
            }
        }
    }
}

impl Default for VoiceState {
    fn default() -> Self {
        Self::new()
    }
}
