use super::*;

/// II.12 admits no exception for "it's only a notification": an unapproved hook does not
/// run. It warns rather than failing, because the machine's state is already correct.
#[tokio::test]
async fn an_unapproved_hook_does_not_run() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("ran");
    write_repo_hook(tmp.path(), Event::OnDrift, &touch_script(&marker));
    let hooks = EventHooks::load(&config_at(tmp.path()));
    hooks.fire(Event::OnDrift, json!({})).await;
    assert!(!marker.exists(), "an unapproved hook ran");
}
