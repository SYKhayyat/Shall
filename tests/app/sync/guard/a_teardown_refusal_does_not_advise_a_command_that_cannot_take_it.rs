use super::*;

#[tokio::test]
async fn a_teardown_refusal_does_not_advise_a_command_that_cannot_take_it() {
    // `shall unmanage` takes a package line. Offering it for a `link:` teardown names a
    // command that cannot accept the thing the refusal is about.
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(1);
    let err = enforce_extras(
        &cfg,
        &reg,
        &extras(&["link:/a", "link:/b"]),
        &Reaping::new(),
        GuardScope::Sync,
    )
    .await
    .expect_err("two removals over a limit of one must be refused");
    let msg = err.to_string();
    assert!(msg.contains("managed resources"), "{}", msg);
    assert!(!msg.contains("shall unmanage"), "{}", msg);
    assert!(msg.contains("shall plan"), "{}", msg);
}
