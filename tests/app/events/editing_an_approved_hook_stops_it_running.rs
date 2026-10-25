use super::*;

/// An edited hook is unapproved again — the ledger's actual job.
#[tokio::test]
async fn editing_an_approved_hook_stops_it_running() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("ran");
    write_repo_hook(tmp.path(), Event::OnDrift, &exit_with(0));
    EventHooks::load(&config_at(tmp.path()))
        .approve_all()
        .unwrap();

    write_repo_hook(tmp.path(), Event::OnDrift, &touch_script(&marker));
    EventHooks::load(&config_at(tmp.path()))
        .fire(Event::OnDrift, json!({}))
        .await;
    assert!(!marker.exists(), "an edited hook ran on the old approval");
}
