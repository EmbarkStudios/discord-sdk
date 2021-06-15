#![allow(dead_code)]

pub use discord_sdk as ds;
pub use tokio::sync::mpsc;

/// Application ID for the "game" in this case, "Andy's test app" which is the
/// same application used in the Discord Game SDK's own examples
pub const APP_ID: ds::AppId = 310270644849737729;

pub fn init_logger() {
    let _ = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .with_test_writer()
        .try_init();
}

#[derive(Debug)]
pub enum Msg {
    Event(ds::Event),
    Error(ds::Error),
}

pub struct Client {
    pub discord: ds::Discord,
    pub user: ds::user::User,
    pub events: mpsc::UnboundedReceiver<Msg>,
}

struct Forward(mpsc::UnboundedSender<Msg>);

#[async_trait::async_trait]
impl ds::DiscordHandler for Forward {
    async fn on_event(&self, event: ds::Event) {
        let _ = self.0.send(Msg::Event(event));
    }

    async fn on_error(&self, error: ds::Error) {
        let _ = self.0.send(Msg::Error(error));
    }
}

pub async fn make_client(subs: ds::Subscriptions) -> Result<Client, ds::Error> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let discord = ds::Discord::new(ds::DiscordApp::PlainId(APP_ID), subs, Box::new(Forward(tx)))?;

    tracing::info!("waiting for handshake...");
    let user = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    if let Msg::Event(ds::Event::Ready { user, .. }) = msg {
                        break user;
                    }
                }
                None => panic!("discord closed"),
            }
        }
    })
    .await?;

    Ok(Client {
        discord,
        user,
        events: rx,
    })
}

pub struct DualClients {
    pub one: Client,
    pub two: Client,
}

/// Creates 2 clients, each connected to a different Discord application, this
/// requires that you have started and logged in to 2 different versions of
/// Discord (stable, canary, or PTB)
///
/// See more details [here](https://discord.com/developers/docs/game-sdk/sdk-starter-guide#testing-locally-with-two-clients)
#[cfg(feature = "local-testing")]
pub async fn make_dual_clients(subs: ds::Subscriptions) -> Result<DualClients, ds::Error> {
    std::env::set_var("DISCORD_INSTANCE_ID", 0.to_string());
    let one = make_client(subs).await?;

    std::env::set_var("DISCORD_INSTANCE_ID", 1.to_string());
    let two = make_client(subs).await?;

    Ok(DualClients { one, two })
}
