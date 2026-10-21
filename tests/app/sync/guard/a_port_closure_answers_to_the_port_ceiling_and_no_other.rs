use super::*;

/// **`N8`: a port closure is a removal, and it answers to its own ceiling.** Under `Y20` it
/// spent `max_extra_removals`, so the run that first declares a perimeter consumed a budget
/// meant for `link:`/`service:` teardowns and refused a resource change it had nothing to
/// do with.
#[tokio::test]
async fn a_port_closure_answers_to_the_port_ceiling_and_no_other() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(3);
    cfg.guard.max_port_closures = 2;
    let reaping = Reaping::new();

    let err = enforce_ports(
        &cfg,
        &reg,
        &ports(&["22/tcp", "80/tcp", "443/tcp"]),
        &reaping,
        GuardScope::Sync,
    )
    .await
    .expect_err("three closures over a limit of two must be refused");
    let msg = err.to_string();
    assert!(msg.contains("closes 3 ports"), "{msg}");
    assert!(
        msg.contains("max_port_closures"),
        "the refusal must name the setting the reader changes: {msg}"
    );
    assert!(
        !msg.contains("max_extra_removals") && !msg.contains("max_removals"),
        "a port refusal must not send the reader to another kind's setting: {msg}"
    );
    assert_eq!(
        reaping.so_far(RemovalKind::Port),
        0,
        "refused, so not spent"
    );
}
