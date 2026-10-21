use super::*;

/// **`N8`: the ceiling over everything.** Nine changes pass three ceilings of three and are
/// still nine changes. Off by default, so this is what setting it buys.
#[tokio::test]
async fn the_total_catches_what_every_per_kind_ceiling_passes() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(3);
    cfg.guard.max_total_changes = 8;
    let reaping = Reaping::new();

    assert!(enforce(
        &cfg,
        &reg,
        &pairs(&["a", "b", "c"]),
        &reaping,
        GuardScope::Sync
    )
    .await
    .is_ok());
    assert!(enforce_extras(
        &cfg,
        &reg,
        &extras(&["link:/a", "link:/b", "link:/c"]),
        &reaping,
        GuardScope::Sync
    )
    .await
    .is_ok());
    let err = enforce_ports(
        &cfg,
        &reg,
        &ports(&["22/tcp", "80/tcp", "443/tcp"]),
        &reaping,
        GuardScope::Sync,
    )
    .await
    .expect_err("nine changes over a total of eight must be refused");
    let msg = err.to_string();
    assert!(msg.contains("makes 9 changes in total"), "{msg}");
    assert!(msg.contains("max_total_changes"), "{msg}");
    assert!(
        !msg.contains("max_port_closures"),
        "three ports are inside the port ceiling â€” naming it would send the reader to a \
             number that is not the problem: {msg}"
    );
}
