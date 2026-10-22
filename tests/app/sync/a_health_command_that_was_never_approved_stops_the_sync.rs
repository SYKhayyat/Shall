use super::*;

/// **A health *command* that was never approved stops the sync.**
///
/// Found by the mutation gate: `require_health_commands_approved` replaced with `Ok(())`
/// survived the whole suite, and so did the `!` that made the ledger's verdict answer. **It is
/// the check that keeps a machine from running a command nobody approved**: a health check
/// written as `cmd:…` is code Shall will run, and the II.12 ledger is the one place that
/// says which code is allowed to run.
///
/// Port probes run no code and are never gated; **a command is**, and the refusal names the
/// ledger entry that would approve it.
#[tokio::test]
async fn a_health_command_that_was_never_approved_stops_the_sync() {
    let (engine, _mock) = engine(config_with_health("unapproved", &["cmd:echo hello"])).await;

    let refused = engine.require_health_commands_approved(&no_changes());
    assert!(
        refused.is_err(),
        "a health command nobody approved was allowed to start: {refused:?}"
    );
    let message = refused.unwrap_err().to_string();
    assert!(
        message.contains("echo hello"),
        "the refusal must name the command that was not approved: {message}"
    );
    assert!(
        message.contains("shall lock"),
        "the refusal must name the command that would approve it: {message}"
    );
}

/// **And a port probe is not a command**, so it is not gated by the ledger.
#[tokio::test]
async fn a_port_probe_is_not_gated_by_the_approval_ledger() {
    let (engine, _mock) = engine(config_with_health("port-not-gated", &["port:8080"])).await;

    assert!(
        engine
            .require_health_commands_approved(&no_changes())
            .is_ok(),
        "a port probe runs no code, so the approval ledger has nothing to say about it"
    );
}
