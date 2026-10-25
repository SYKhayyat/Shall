use super::*;

/// `everything` minus a kind, and minus a whole group.
#[test]
fn an_exclusion_subtracts_from_whatever_was_named() {
    let no_exec = LockSelection::parse("everything", &words(&["exec"])).unwrap();
    assert!(!no_exec.includes(LockKind::Exec));
    assert_eq!(kinds_of(&no_exec).len(), ALL.len() - 1);

    let no_scripts = LockSelection::parse("everything", &words(&["scripts"])).unwrap();
    assert_eq!(kinds_of(&no_scripts), PACKAGES.to_vec());

    let listed = LockSelection::parse("packages", &words(&["backends"])).unwrap();
    assert_eq!(kinds_of(&listed), vec![LockKind::Versions]);
}
