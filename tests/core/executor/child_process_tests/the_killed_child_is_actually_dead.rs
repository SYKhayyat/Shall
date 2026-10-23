#[cfg(unix)]
use super::*;

/// A bound that reports the hang but leaves the process running has moved the leak, not
/// closed it — the wedged `Checkpoint-Computer` held its restore point for 76 minutes.
#[cfg(unix)]
#[tokio::test]
async fn the_killed_child_is_actually_dead() {
    let layer = RawExecutor::with_idle(ChildStdin::Closed, Some(std::time::Duration::from_secs(2)));
    // `pgrep -f` on a marker only this test uses; a survivor is visible to the whole host.
    let marker = format!("shall-idle-probe-{}", std::process::id());
    let args = vec!["-c".to_string(), format!("# {}\nsleep 600", marker)];
    let _ = layer.execute("sh", &args, &HashMap::new()).await;
    let found = std::process::Command::new("pgrep")
        .args(["-f", &marker])
        .output();
    if let Ok(found) = found {
        assert!(
            String::from_utf8_lossy(&found.stdout).trim().is_empty(),
            "the child outlived the bound that killed it"
        );
    }
}
