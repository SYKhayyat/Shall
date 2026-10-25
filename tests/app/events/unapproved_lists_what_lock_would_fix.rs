use super::*;

/// `check` surfaces the hooks that will silently not run. Before `lock`, every declared
/// hook is unapproved; after, none is.
#[test]
fn unapproved_lists_what_lock_would_fix() {
    let tmp = tempfile::tempdir().unwrap();
    write_repo_hook(tmp.path(), Event::OnDrift, "#!/bin/sh\ntrue\n");
    let hooks = EventHooks::load(&config_at(tmp.path()));

    assert_eq!(
        hooks.unapproved().len(),
        1,
        "an unlocked hook is unapproved"
    );
    hooks.approve_all().unwrap();
    assert!(
        EventHooks::load(&config_at(tmp.path()))
            .unapproved()
            .is_empty(),
        "lock should approve it"
    );
}
