use super::*;

/// U15's ruling, and the reason it was a ruling: both locations, both run. A precedence
/// rule would mean adding a local notification silently disables the shared policy.
#[test]
fn a_repo_hook_and_a_machine_hook_for_one_event_both_load() {
    let tmp = tempfile::tempdir().unwrap();
    write_repo_hook(tmp.path(), Event::OnDrift, "#!/bin/sh\necho repo\n");
    let mut config = config_at(tmp.path());
    config.events.insert(
        "on_drift".to_string(),
        "#!/bin/sh\necho machine\n".to_string(),
    );

    let hooks = EventHooks::load(&config);
    let drift: Vec<&EventHook> = hooks
        .all()
        .iter()
        .filter(|h| h.event == Event::OnDrift)
        .collect();
    assert_eq!(drift.len(), 2, "{:?}", hooks.all());
    // Repo first: the shared policy acts, then this machine is told.
    assert_eq!(drift[0].origin, "hooks/on_drift");
    assert_eq!(drift[1].origin, "preferences.toml");
}
