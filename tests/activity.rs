mod shared;

#[cfg(feature = "local-testing")]
#[tokio::test]
async fn test_activity() {
    use shared::ds::{self, activity};

    shared::init_logger();

    let dual = shared::make_dual_clients(ds::Subscriptions::ACTIVITY)
        .await
        .expect("failed to start clients");

    let shared::DualClients { one, two } = dual;

    let mut events = one.events;
    tokio::task::spawn(async move {
        while let Some(event) = events.recv().await {
            tracing::info!(which = 1, event = ?event);
        }
    });

    let (invite_tx, invite_rx) = tokio::sync::oneshot::channel();
    let (join_tx, join_rx) = tokio::sync::oneshot::channel();

    let mut events = two.events;
    tokio::task::spawn(async move {
        let mut invite_tx = Some(invite_tx);
        let mut join_tx = Some(join_tx);
        while let Some(event) = events.recv().await {
            tracing::info!(which = 2, event = ?event);

            match event {
                shared::Msg::Event(ds::Event::ActivityInvite(invite)) => {
                    if let Some(tx) = invite_tx.take() {
                        tx.send(invite).unwrap();
                    }
                }
                shared::Msg::Event(ds::Event::ActivityJoin { secret }) => {
                    if let Some(tx) = join_tx.take() {
                        tx.send(secret).unwrap();
                    }
                }
                _ => {}
            }
        }
    });

    let _one_user = one.user;
    let two_user = two.user;

    let one = one.discord;
    let two = two.discord;

    let party_id = "partyfun123";
    tracing::info!(
        "1 => updating activity: {:#?}",
        one.update_activity(
            activity::ActivityBuilder::new()
                .state("test 1")
                .details("much presence very details")
                .party(
                    party_id,
                    std::num::NonZeroU32::new(1),
                    std::num::NonZeroU32::new(2),
                    activity::PartyPrivacy::Public,
                )
                .secrets(activity::Secrets {
                    join: Some("muchsecretverysecurity".to_owned()),
                    ..Default::default()
                }),
        )
        .await
        .expect("failed to update presence")
    );

    // wait a few seconds on windows, just to see if it's because of slow I/O?
    #[cfg(windows)]
    {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    tracing::info!("1 => inviting {}", two_user);
    one.invite_user(
        two_user.id,
        "please join or the test will fail",
        activity::ActivityActionKind::Join,
    )
    .await
    .expect("failed to send invite");

    tracing::info!("2 => waiting for invite");
    let invite = tokio::time::timeout(std::time::Duration::from_secs(5), invite_rx)
        .await
        .expect("timed out waiting for invite")
        .expect("event task dropped");

    two.accept_invite(&invite).await.unwrap();

    tracing::info!("2 => waiting for join");
    let secret = tokio::time::timeout(std::time::Duration::from_secs(5), join_rx)
        .await
        .expect("timed out waiting for join")
        .expect("event task dropped");

    assert_eq!(secret, "muchsecretverysecurity",);

    one.disconnect().await;
    two.disconnect().await;
}
