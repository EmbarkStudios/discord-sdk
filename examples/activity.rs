use discord_sdk as ds;

#[path = "shared._rs"]
mod shared;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (client, user) = shared::make_client(ds::Subscriptions::ACTIVITY).await;

    tracing::info!("connected to Discord, local user is {:#?}", user);

    let rp = ds::ActivityBuilder::default()
        .details("Fruit Tarts".to_owned())
        .state("Pop Snacks".to_owned())
        .assets(
            ds::Assets::default()
                .large("the".to_owned(), Some("u mage".to_owned()))
                .small("the".to_owned(), Some("i mage".to_owned())),
        );

    tracing::info!("updated activity: {:?}", client.update_presence(rp).await);

    let mut r = String::new();
    let _ = std::io::stdin().read_line(&mut r);

    tracing::info!("cleared activity: {:?}", client.clear_presence().await);

    client.disconnect().await;

    Ok(())
}
