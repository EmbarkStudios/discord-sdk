macro_rules! handle_response {
    ($oneshot:expr, $bind:pat => $arm:block) => {
        match $oneshot.await?? {
            $bind => $arm,
            other => unreachable!("response {:?} should be impossible", other),
        }
    };
}
