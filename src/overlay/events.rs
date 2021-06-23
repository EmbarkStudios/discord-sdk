use super::Visibility;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UpdateEvent {
    /// Whether the user has the overlay enabled or disabled. If the overlay
    /// is disabled, all the functionality of the SDK will still work. The
    /// calls will instead focus the Discord client and show the modal there
    /// instead of in application.
    pub enabled: bool,
    /// Whether the overlay is visible or not.
    #[serde(rename = "locked")]
    pub visible: Visibility,
}

#[derive(Debug)]
pub enum OverlayEvent {
    Update(UpdateEvent),
}
