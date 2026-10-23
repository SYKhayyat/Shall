use super::*;

/// The old keepalive returned a bare `JoinHandle`, which detaches on drop — so nothing a
/// caller did could stop the loop.
#[tokio::test]
async fn dropping_the_keepalive_guard_stops_the_loop() {
    let e = CommandExecutor::new(false, false);
    let keep = e.start_sudo_keepalive().await;
    if !keep.is_running() {
        return; // root, Windows: there is no timestamp to refresh
    }
    let handle = keep.0.as_ref().map(|h| h.abort_handle()).unwrap();
    drop(keep);
    for _ in 0..200 {
        if handle.is_finished() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("the keepalive outlived its guard");
}
