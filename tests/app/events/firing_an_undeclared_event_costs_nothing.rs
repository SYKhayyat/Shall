use super::*;

/// Firing an event nobody listens to must not read the ledger or start a process — that is
/// what makes it safe to fire events at every interesting moment.
#[tokio::test]
async fn firing_an_undeclared_event_costs_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let hooks = EventHooks::load(&config_at(tmp.path()));
    hooks.fire(Event::OnDrift, json!({"removed": ["jq"]})).await;
    // No `locks/` was created, which is the observable proof the ledger was never opened.
    assert!(!tmp.path().join("locks").exists());
}
