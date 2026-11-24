use super::*;
use shall::core::executor::DryRunOutput;
use shall::core::journal::JournalAction;

#[tokio::test]
async fn recovery_carries_past_a_failure_without_rolling_back_the_success() {
    let (engine, mock) = engine(Config::sandboxed(&config_dir("heal-partial"))).await;
    {
        let mut journal = engine.journal.lock().await;
        journal
            .record_start(JournalAction::Install(package("brew", "healed-brew")))
            .expect("the brew recovery entry");
        journal
            .record_start(JournalAction::Install(package("cargo", "healed-cargo")))
            .expect("the cargo recovery entry");
    }
    mock.set_response(
        "brew install -- healed-brew",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "cargo install -- healed-cargo",
        Err(shall::core::Error::command_failed_permanently(
            "the compiler failed",
        )),
    );

    let (logs, guard) = log_collector();
    let result = engine.heal(&[]).await;
    drop(guard);

    assert!(result.is_err(), "the failed recovery did not fail the run");

    // The count is the whole claim: one operation failed and one was recovered, so the log must
    // say one. Nothing pinned this line — deleting the `!` in `if !failed.is_empty()` survived
    // the whole suite, because the only effect of the inversion is that a run with failures says
    // nothing about them.
    let logs = logs.lock().expect("the log mutex").clone();
    let said = logs.join("\n");
    assert!(
        said.contains("1 interrupted operation(s) did not recover"),
        "the recovery did not report which operations it could not repair: {said}"
    );
    let calls = mock.get_calls().await;
    assert!(
        calls
            .iter()
            .any(|call| call == "brew install -- healed-brew"),
        "recovery stopped before the successful operation: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|call| call == "cargo install -- healed-cargo"),
        "the failed operation was never attempted: {calls:?}"
    );
    assert!(
        !calls.iter().any(|call| call.contains("uninstall")),
        "recovery rolled back an operation that had already succeeded: {calls:?}"
    );
    let state = engine.state.lock().await;
    assert!(
        state.is_managed("brew", "healed-brew"),
        "the successful recovery was not recorded"
    );
}
