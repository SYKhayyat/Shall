use super::*;

/// Every scope names itself, in both vocabularies, and no two share an answer.
///
/// **This is the assertion that would have caught the dead round trip.** The firewall
/// teardown converted its scope to a string and back through two functions whose
/// vocabularies did not overlap: the producer emitted `"an unattended watch tick"` and the
/// consumer matched `"watch"`, so both named arms were unreachable and every teardown â€”
/// including `N7`'s unattended tick, which reverts by default with nobody watching â€” was
/// guarded and reported as `sync`. The scope is passed as the enum now, and this holds the
/// two labels apart so a catch-all arm cannot quietly reintroduce the collapse.
#[test]
fn every_scope_names_itself_and_no_two_answer_alike() {
    let all = [
        GuardScope::Apply,
        GuardScope::Sync,
        GuardScope::RemoveOrphans,
        GuardScope::PurgeUndeclared,
        GuardScope::Watch,
        GuardScope::Upgrade,
        GuardScope::Canary,
        GuardScope::Remove,
        GuardScope::ShellExit,
        GuardScope::ExpirySweep,
        GuardScope::Heal,
        GuardScope::Rebuild,
    ];

    let mut commands = std::collections::BTreeSet::new();
    let mut prose = std::collections::BTreeSet::new();
    for scope in all {
        assert!(
            commands.insert(scope.as_str()),
            "{:?} shares `as_str` with another scope, so a refusal tells the user to retype \
                 a different command",
            scope
        );
        assert!(
            prose.insert(scope.during()),
            "{:?} shares `during` with another scope â€” a catch-all arm has collapsed them, \
                 which is how the label this replaced answered `sync` for nine of twelve",
            scope
        );
    }
    assert_eq!(commands.len(), all.len());
    assert_eq!(prose.len(), all.len());

    // The one distinction the whole mechanism exists for.
    assert_eq!(GuardScope::Watch.during(), "an unattended watch tick");
    assert_eq!(GuardScope::Sync.during(), "sync");
}
