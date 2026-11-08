use super::*;
use shall::core::executor::DryRunOutput;
use shall::core::journal::JournalAction;

/// A recovery changes the registry, and `heal` is a whole command — nothing runs after it to
/// write what it changed. The save gate is `!recovered.is_empty() || !kept.is_empty() ||
/// !failed.is_empty()`, and each arm has to be enough on its own: a recovered-only run is the
/// common case, and the mutation job reports both `||` → `&&` and deleting the `!` on
/// `recovered` as survivors because no test drove a run whose *only* non-empty set was that one.
#[tokio::test]
async fn a_heal_that_recovered_an_install_writes_it_to_disk() {
    let dir = config_dir("heal-persist");
    let state_path = dir.join("state.json");
    let (engine, mock) = engine(Config::sandboxed(&dir)).await;
    {
        let mut journal = engine.journal.lock().await;
        journal
            .record_start(JournalAction::Install(package("brew", "healed-brew")))
            .expect("the recovery entry");
    }
    mock.set_response(
        "brew install -- healed-brew",
        Ok(DryRunOutput::default().into()),
    );
    let (logs, guard) = log_collector();

    engine.heal(&[]).await.expect("the recovery failed");
    drop(guard);

    let written = std::fs::read_to_string(&state_path)
        .expect("the registry was not written after a recovery, so the orphan comes back");
    assert!(
        written.contains("healed-brew"),
        "the recovered package is missing from the registry on disk: {written}"
    );
    let logs = logs.lock().expect("the log mutex").clone();
    assert!(
        logs.iter()
            .any(|line| line.contains("recovered 1 operation(s)")),
        "the recovery was not reported: {logs:?}"
    );
}
