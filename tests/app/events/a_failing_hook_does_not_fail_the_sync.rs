use super::*;

/// A hook that exits non-zero warns and does not fail anything — 7j's exit condition. The
/// sync's job is the machine's state, and a webhook that is down does not change it.
#[tokio::test]
async fn a_failing_hook_does_not_fail_the_sync() {
    let tmp = tempfile::tempdir().unwrap();
    write_repo_hook(tmp.path(), Event::AfterSync, &exit_with(9));
    let hooks = EventHooks::load(&config_at(tmp.path()));
    hooks.approve_all().unwrap();
    // `fire` returns nothing to propagate: a hook has no way to fail its caller by
    // construction, not by a caller remembering to ignore it.
    hooks.fire(Event::AfterSync, json!({})).await;
}
