use super::*;

/// The two locations are separately approved. Approving the shared policy must not
/// rubber-stamp whatever this machine has in `preferences.toml`, or the ledger's whole
/// point — *is this the script I agreed to run* — is gone.
#[test]
fn the_two_locations_have_different_ledger_identities() {
    let tmp = tempfile::tempdir().unwrap();
    // The same bytes in both places, which is the case that would expose a shared identity.
    let body = "#!/bin/sh\ntrue\n";
    write_repo_hook(tmp.path(), Event::AfterSync, body);
    let mut config = config_at(tmp.path());
    config
        .events
        .insert("after_sync".to_string(), body.to_string());

    let hooks = EventHooks::load(&config);
    assert_eq!(hooks.all().len(), 2);
    assert_ne!(hooks.all()[0].id, hooks.all()[1].id);
}
