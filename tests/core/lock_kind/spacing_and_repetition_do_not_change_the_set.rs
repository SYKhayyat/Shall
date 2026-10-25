use super::*;

/// Whitespace around a comma is a person typing, not an error. And a repeated word is not
/// a duplicate entry — the set is a set.
#[test]
fn spacing_and_repetition_do_not_change_the_set() {
    assert_eq!(
        kinds_of(&sel(" exec , hooks ")),
        kinds_of(&sel("hooks,exec"))
    );
    assert_eq!(kinds_of(&sel("exec,exec")), vec![LockKind::Exec]);
    assert_eq!(kinds_of(&sel("packages,versions")), PACKAGES.to_vec());
}
