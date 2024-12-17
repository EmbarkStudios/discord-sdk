use crate::{
    activity::events::ActivityEvent,
    handler::DiscordMsg,
    overlay::events::OverlayEvent,
    proto::event::ClassifiedEvent,
    relations::events::RelationshipEvent,
    user::{events::UserEvent, User},
};
use tokio::sync::{broadcast, watch};

/// An event wheel, with a different `spoke` per class of events
pub struct Wheel {
    activity: broadcast::Sender<ActivityEvent>,
    relations: broadcast::Sender<RelationshipEvent>,

    user: watch::Receiver<UserState>,
    overlay: watch::Receiver<OverlayState>,
}

impl Wheel {
    pub fn new(error: Box<dyn OnError>) -> (Self, WheelHandler) {
        let (activity_tx, _activity_rx) = broadcast::channel(10);
        let (rl_tx, _rl_rx) = broadcast::channel(10);

        let (user_tx, user_rx) =
            watch::channel(UserState::Disconnected(crate::Error::NoConnection));
        let (overlay_tx, overlay_rx) = watch::channel(OverlayState {
            enabled: false,
            visible: crate::overlay::Visibility::Hidden,
        });

        (
            Self {
                activity: activity_tx.clone(),
                relations: rl_tx.clone(),
                user: user_rx,
                overlay: overlay_rx,
            },
            WheelHandler {
                activity: activity_tx,
                relations: rl_tx,
                user: user_tx,
                overlay: overlay_tx,
                error,
            },
        )
    }

    #[inline]
    pub fn activity(&self) -> ActivitySpoke {
        ActivitySpoke(self.activity.subscribe())
    }

    #[inline]
    pub fn relationships(&self) -> RelationshipSpoke {
        RelationshipSpoke(self.relations.subscribe())
    }

    #[inline]
    pub fn user(&self) -> UserSpoke {
        UserSpoke(self.user.clone())
    }

    #[inline]
    pub fn overlay(&self) -> OverlaySpoke {
        OverlaySpoke(self.overlay.clone())
    }
}

pub struct ActivitySpoke(pub broadcast::Receiver<ActivityEvent>);
pub struct RelationshipSpoke(pub broadcast::Receiver<RelationshipEvent>);
pub struct UserSpoke(pub watch::Receiver<UserState>);
pub struct OverlaySpoke(pub watch::Receiver<OverlayState>);

#[async_trait::async_trait]
pub trait OnError: Send + Sync {
    async fn on_error(&self, _error: crate::Error) {}
}

#[async_trait::async_trait]
impl<F> OnError for F
where
    F: Fn(crate::Error) + Send + Sync,
{
    async fn on_error(&self, error: crate::Error) {
        self(error);
    }
}

#[derive(Debug)]
pub enum UserState {
    Connected(User),
    Disconnected(crate::Error),
}

#[derive(Debug)]
pub struct OverlayState {
    /// Whether the user has the overlay enabled or disabled. If the overlay
    /// is disabled, all the functionality of the SDK will still work. The
    /// calls will instead focus the Discord client and show the modal there
    /// instead of in application.
    pub enabled: bool,
    /// Whether the overlay is visible or not.
    pub visible: crate::overlay::Visibility,
}

/// The write part of the [`Wheel`] which is used by the actual handler task
pub struct WheelHandler {
    activity: broadcast::Sender<ActivityEvent>,
    relations: broadcast::Sender<RelationshipEvent>,

    user: watch::Sender<UserState>,
    overlay: watch::Sender<OverlayState>,

    error: Box<dyn OnError>,
}

#[async_trait::async_trait]
impl super::DiscordHandler for WheelHandler {
    async fn on_message(&self, msg: DiscordMsg) {
        match msg {
            DiscordMsg::Error(err) => self.error.on_error(err).await,
            DiscordMsg::Event(eve) => match ClassifiedEvent::from(eve) {
                ClassifiedEvent::User(user) => {
                    let us = match user {
                        UserEvent::Connect(eve) => UserState::Connected(eve.user),
                        UserEvent::Update(eve) => UserState::Connected(eve.user),
                        UserEvent::Disconnect(de) => UserState::Disconnected(de.reason),
                    };

                    if let Err(e) = self.user.send(us) {
                        tracing::warn!(error = %e, "User event was unobserved");
                    }
                }
                ClassifiedEvent::Relations(re) => {
                    if let Err(e) = self.relations.send(re) {
                        tracing::warn!(event = ?e.0, "Relationship event was unobserved");
                    }
                }
                ClassifiedEvent::Activity(activity) => {
                    if let Err(e) = self.activity.send(activity) {
                        tracing::warn!(event = ?e.0, "Activity event was unobserved");
                    }
                }
                ClassifiedEvent::Overlay(overlay) => {
                    let os = match overlay {
                        OverlayEvent::Update(update) => OverlayState {
                            enabled: update.enabled,
                            visible: update.visible,
                        },
                    };

                    if let Err(e) = self.overlay.send(os) {
                        tracing::warn!(error = %e, "Overlay event was unobserved");
                    }
                }
            },
        }
    }
}
