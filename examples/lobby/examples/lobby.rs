use ds::lobby::{self, search};
use examples_shared::{self as es, anyhow, ds, tokio, tracing};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = es::make_client(ds::Subscriptions::LOBBY).await;

    let mut lobby_events = client.wheel.lobby();

    tokio::task::spawn(async move {
        while let Ok(le) = lobby_events.0.recv().await {
            tracing::info!(event = ?le, "received lobby event");
        }
    });

    let lobby = client
        .discord
        .create_lobby(
            lobby::CreateLobbyBuilder::new()
                .add_metadata(std::iter::once(("crab".to_owned(), "confirmed".to_owned())))
                .capacity(std::num::NonZeroU32::new(2))
                .kind(lobby::LobbyKind::Public),
        )
        .await?;

    tracing::info!("created lobby: {:#?}", lobby);

    tracing::info!("waiting for a while before searching lobbies");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Search for the lobbies that are owned by us
    let lobbies = client
        .discord
        .search_lobbies(
            search::SearchQuery::default()
                .add_filter(
                    search::SearchKey::OwnerId,
                    search::LobbySearchComparison::Equal,
                    search::SearchValue::string(client.user.id.0.to_string()),
                )
                .distance(search::LobbySearchDistance::Global),
        )
        .await?;

    tracing::info!("found lobbies: {:#?}", lobbies);

    tracing::debug!(
        "connected to lobby voice: {:?}",
        client.discord.connect_lobby_voice(lobby.id).await
    );

    let mut r = String::new();
    let _ = std::io::stdin().read_line(&mut r);

    tracing::info!(
        "deleted lobby: {:?}",
        client.discord.delete_lobby(lobby.id).await
    );

    client.discord.disconnect().await;

    Ok(())
}
