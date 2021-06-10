pub use discord_sdk as ds;

pub const APP_ID: ds::AppId = 310270644849737729;

struct Printer(tokio::sync::mpsc::Sender<Option<ds::User>>);

#[async_trait::async_trait]
impl ds::DiscordHandler for Printer {
    async fn on_event(&self, event: ds::Event) {
        println!("received event form discord: {:#?}", event);

        match event {
            ds::Event::Ready { user, .. } => {
                self.0.send(Some(user)).await;
            }
            ds::Event::Disconnected { .. } => {
                self.0.send(None).await;
            }
            _ => {}
        }
    }

    async fn on_error(&self, error: ds::Error) {
        eprintln!("an error occurred! {:#?}", error);
    }
}

pub fn make_client(tx: tokio::sync::mpsc::Sender<Option<ds::User>>) -> ds::Discord {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .init();

    ds::Discord::new(
        ds::DiscordApp::PlainId(APP_ID),
        ds::Subscriptions::ACTIVITY,
        Box::new(Printer(tx)),
    )
    .expect("unable to create discord client")
}
