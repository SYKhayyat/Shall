use super::*;

/// `0` in the config means the old behaviour, for whoever has a legitimately silent
/// half-hour command and would rather wait than tune a number.
#[tokio::test]
async fn no_bound_means_no_bound() {
    let layer = RawExecutor::with_idle(ChildStdin::Closed, None);
    let (cmd, args) = silent_forever();
    let ran = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        layer.execute(cmd, &args, &HashMap::new()),
    )
    .await;
    assert!(ran.is_err(), "an unbounded layer must still be waiting");
}
