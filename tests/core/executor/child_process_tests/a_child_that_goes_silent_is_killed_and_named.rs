use super::*;

/// The bug: `shall uninstall choco:bat` sat 76 minutes on a `Checkpoint-Computer` that had
/// already written its restore point, because nothing outside the DAG bounded a child at
/// all. Two earlier hangs were killed by hand and recorded as undiagnosed.
#[tokio::test]
async fn a_child_that_goes_silent_is_killed_and_named() {
    let layer = RawExecutor::with_idle(ChildStdin::Closed, Some(std::time::Duration::from_secs(2)));
    let (cmd, args) = silent_forever();
    let started = std::time::Instant::now();
    let err = layer
        .execute(cmd, &args, &HashMap::new())
        .await
        .expect_err("a child silent past the bound must not be waited on forever");
    let waited = started.elapsed();

    assert!(
        waited < std::time::Duration::from_secs(45),
        "waited {:?}, so the bound did not fire",
        waited
    );
    let message = err.to_string();
    assert!(
        message.contains("produced no output"),
        "the error must say what happened: {}",
        message
    );
    // Naming the program alone does not identify a hang on a host running six of them.
    assert!(
        message.contains(cmd) && args.iter().all(|a| message.contains(a.as_str())),
        "the error must name the command that hung: {}",
        message
    );
    assert!(
        message.contains("command_idle_timeout_secs"),
        "the error must name the dial that changes it: {}",
        message
    );
}
