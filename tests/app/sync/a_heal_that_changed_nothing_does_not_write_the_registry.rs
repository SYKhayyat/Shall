use super::*;
use shall::core::journal::JournalAction;

/// The other side of the save gate, and the side that makes it a gate at all: a heal whose
/// three sets are all empty must write nothing, and must not print the summaries of things
/// that did not happen. Deleting the `!` from any of the three arms turns its guard true on an
/// empty set, so this run would write an unchanged registry and report "recovered 0
/// operation(s)" — the exact drift the mutation job kept naming.
///
/// An interrupted install for a manager this machine does not have is the fixture: `heal` can
/// account for the entry (it lands in `unreachable`, which is not one of the three save arms)
/// and can touch nothing, so `recovered`, `kept` and `failed` all stay empty while the command
/// still does real work.
#[tokio::test]
async fn a_heal_that_changed_nothing_does_not_write_the_registry() {
    let dir = config_dir("heal-noop");
    let state_path = dir.join("state.json");
    let (engine, _mock) = engine(Config::sandboxed(&dir)).await;
    {
        let mut journal = engine.journal.lock().await;
        journal
            .record_start(JournalAction::Install(package("nonesuch", "ghost")))
            .expect("the unreachable entry");
    }
    let (logs, guard) = log_collector();

    let result = engine.heal(&[]).await;
    drop(guard);

    assert!(
        result.is_err(),
        "an operation whose manager is not on this machine was reported as recovered"
    );
    assert!(
        !state_path.exists(),
        "a heal that changed nothing created a registry file"
    );
    let logs = logs.lock().expect("the log mutex").clone();
    for impossible in [
        "recovered 0 operation(s)",
        "kept 0 protected",
        "0 interrupted script(s)",
    ] {
        assert!(
            !logs.iter().any(|line| line.contains(impossible)),
            "a heal with nothing in that set said {impossible:?} anyway: {logs:?}"
        );
    }
}
