use super::*;

/// The managers a run named on the bare lock, so a typo can be refused before it scopes the
/// run to nothing (`Q9`). Both the include and the exclude side, since either can carry one.
#[test]
fn every_manager_named_on_the_bare_lock_is_reported_for_checking() {
    let s = LockSelection::parse("packages:apt", &words(&["backends:cargo"])).unwrap();
    let mut named = s.managers_named();
    named.sort();
    named.dedup();
    assert_eq!(named, vec!["apt".to_string(), "cargo".to_string()]);

    // An exclusion naming a kind that was never selected is a no-op, so its manager is not
    // reported either — there is nothing for the registry check to be checking on behalf of.
    let unrelated = LockSelection::parse("backends:apt", &words(&["versions:cargo"])).unwrap();
    assert_eq!(unrelated.managers_named(), vec!["apt".to_string()]);

    // **A version pin's manager is not this host's business.** `locks/versions.json` travels
    // with the config repo, so `versions:apt` is an ordinary scope on a machine that has
    // never run apt, and handing it to the registry check would refuse it there.
    assert!(sel("versions:apt").managers_named().is_empty());

    // A hook name is not a manager and must not be handed to the registry check.
    let hooks = sel("hooks:after_install");
    assert!(hooks.managers_named().is_empty());
}
