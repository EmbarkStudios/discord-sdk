use discord_sdk as ds;

#[path = "shared._rs"]
mod shared;

use ds::lobby;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (client, user) = shared::make_client(ds::Subscriptions::LOBBY).await;

    let lobby = client
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
        .search_lobbies(
            lobby::SearchQuery::default()
                .add_filter(
                    lobby::SearchKey::OwnerId,
                    lobby::LobbySearchComparison::Equal,
                    lobby::SearchValue::string(user.id.0.to_string()),
                )
                .distance(lobby::LobbySearchDistance::Global),
        )
        .await?;

    tracing::info!("found lobbies: {:#?}", lobbies);

    tracing::debug!(
        "connected to lobby voice: {:?}",
        client.connect_lobby_voice(lobby.id).await
    );

    let mut r = String::new();
    let _ = std::io::stdin().read_line(&mut r);

    tracing::info!("deleted lobby: {:?}", client.delete_lobby(lobby.id).await);

    client.disconnect().await;

    Ok(())
}
