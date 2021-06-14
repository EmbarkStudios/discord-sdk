mod shared;

#[cfg(feature = "local-testing")]
#[tokio::test]
async fn test_local_lobbies() {
    use shared::ds::{self, lobby};

    shared::init_logger();

    let dual = shared::make_dual_clients(ds::Subscriptions::LOBBY)
        .await
        .expect("failed to start clients");

    let shared::DualClients { one, two } = dual;

    let mut events = one.events;
    tokio::task::spawn(async move {
        while let Some(event) = events.recv().await {
            tracing::info!(which = 1, event = ?event);
        }
    });

    let mut events = two.events;
    tokio::task::spawn(async move {
        while let Some(event) = events.recv().await {
            tracing::info!(which = 2, event = ?event);
        }
    });

    tracing::info!("1 => creating lobby");
    let lobby = one
        .discord
        .create_lobby(
            lobby::CreateLobbyBuilder::new()
                .capacity(std::num::NonZeroU32::new(2))
                .add_metadata(std::iter::once(("crab".to_owned(), "1".to_owned()))),
        )
        .await
        .expect("failed to create lobby");

    // The SEARCH_LOBBIES command appears to be completely broken so I have filed
    // a bug on it
    // tracing::info!("2 => searching for lobby");
    // let found_lobbies = two
    //     .discord
    //     .search_lobbies(
    //         lobby::SearchQuery::default()
    //             // .add_filter(
    //             //     lobby::SearchKey::OwnerId,
    //             //     lobby::LobbySearchComparison::Equal,
    //             //     lobby::SearchValue::number(one.user.id.0),
    //             // )
    //             .add_filter(
    //                 "crab",
    //                 lobby::LobbySearchComparison::Equal,
    //                 lobby::SearchValue::number(1),
    //             )
    //             .distance(lobby::LobbySearchDistance::Global)
    //             .limit(std::num::NonZeroU32::new(1)),
    //     )
    //     .await
    //     .expect("failed to search lobbies");

    // let found_lobby = found_lobbies.first().expect("failed to find lobby");

    // assert_eq!(lobby.id, found_lobby.id);

    tracing::info!("2 => connecting to lobby");
    let connected_lobby = two
        .discord
        .connect_lobby(lobby::ConnectLobby {
            id: lobby.id,
            secret: lobby.secret.clone(),
        })
        .await
        .expect("failed to connect to lobby");

    assert_eq!(lobby.id, connected_lobby.id);

    let mut md = lobby::Metadata::new();
    md.insert("one".to_owned(), "1".to_owned());
    md.insert("two".to_owned(), "2".to_owned());

    assert!(two
        .discord
        .update_lobby_member(lobby.id, two.user.id, md)
        .await
        .is_ok());

    let lobby_update = one
        .discord
        .get_lobby_update(lobby.id)
        .expect("failed to get lobby update")
        .owner(Some(two.user.id));

    tracing::info!("1 => changing lobby ownership");
    let updated_lobby = one
        .discord
        .update_lobby(lobby_update)
        .await
        .expect("failed to set owner");

    tracing::info!("1 => disconnecting from lobby");
    one.discord
        .disconnect_lobby(lobby.id)
        .await
        .expect("disconnected from lobby");

    // Wait a bit, Discord responds to this quickly but if we try to connect
    // too quickly it will be angry with us since we're "already connected"
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    tracing::info!("1 => connecting to lobby");
    one.discord
        .connect_lobby(lobby::ConnectLobby {
            id: lobby.id,
            secret: lobby.secret.clone(),
        })
        .await
        .expect("connected to lobby");

    tracing::info!("1 => disconnecting from lobby");
    one.discord
        .disconnect_lobby(lobby.id)
        .await
        .expect("disconnected from lobby");

    tracing::info!("2 => deleting lobby");
    two.discord
        .delete_lobby(lobby.id)
        .await
        .expect("deleted lobby");

    one.discord.disconnect().await;
    two.discord.disconnect().await;
}
