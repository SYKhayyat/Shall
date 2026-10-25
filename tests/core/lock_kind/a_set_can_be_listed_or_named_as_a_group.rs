use super::*;

/// The three ways of writing a set, and the same set out of two of them.
#[test]
fn a_set_can_be_listed_or_named_as_a_group() {
    assert_eq!(kinds_of(&sel("everything")), ALL.to_vec());
    assert_eq!(kinds_of(&sel("packages")), PACKAGES.to_vec());
    assert!(sel("exec,hooks").includes(LockKind::Exec));
    assert!(sel("exec,hooks").includes(LockKind::Hooks));
    assert!(!sel("exec,hooks").includes(LockKind::Versions));
    assert_eq!(
        kinds_of(&sel("versions,backends")),
        kinds_of(&sel("packages"))
    );
}
