use ds::relations;
use examples_shared::{self as es, anyhow, ds, tokio, tracing};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = es::make_client(ds::Subscriptions::RELATIONSHIPS).await;

    let mut rel_events = client.wheel.relationships().0;

    let relationships = client.discord.get_relationships().await?;
    tracing::info!("got relationships: {:#?}", relationships);

    let relationships = std::sync::Arc::new(relations::state::Relationships::new(relationships));
    let rs = relationships.clone();

    tokio::task::spawn(async move {
        while let Ok(re) = rel_events.recv().await {
            tracing::info!(event = ?re, "received relationship event");

            rs.on_event(re);
        }
    });

    tokio::task::spawn_blocking(move || {
        let mut r = String::new();
        let _ = std::io::stdin().read_line(&mut r);
    })
    .await
    .expect("failed to spawn task");

    tracing::info!("current relationship states: {:#?}", relationships);

    client.discord.disconnect().await;

    Ok(())
}
