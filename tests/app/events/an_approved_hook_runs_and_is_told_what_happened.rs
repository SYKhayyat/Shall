use super::*;

/// The payload reaches the hook on stdin, and approval is what let it run.
#[tokio::test]
async fn an_approved_hook_runs_and_is_told_what_happened() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("payload.json");
    write_repo_hook(tmp.path(), Event::OnDrift, &capture_stdin_script(&out));
    let hooks = EventHooks::load(&config_at(tmp.path()));
    assert_eq!(hooks.approve_all().unwrap(), 1);

    hooks.fire(Event::OnDrift, json!({"removed": ["jq"]})).await;

    let body = std::fs::read_to_string(&out).expect("the hook ran and read stdin");
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(v["event"], "on_drift");
    assert_eq!(v["data"]["removed"][0], "jq");
}
