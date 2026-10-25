use super::*;

/// An edited hook is unapproved again — the same signal `check` should show after an edit.
#[test]
fn editing_a_hook_makes_it_unapproved_again() {
    let tmp = tempfile::tempdir().unwrap();
    write_repo_hook(tmp.path(), Event::OnDrift, "#!/bin/sh\ntrue\n");
    EventHooks::load(&config_at(tmp.path()))
        .approve_all()
        .unwrap();
    write_repo_hook(tmp.path(), Event::OnDrift, "#!/bin/sh\nfalse\n");
    assert_eq!(
        EventHooks::load(&config_at(tmp.path())).unapproved().len(),
        1
    );
}
