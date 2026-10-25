use super::*;

/// The two variants that *do* withdraw, and the reason each is safe to: both carry the
/// name they looked up, so nothing is inferred from a sentence.
#[test]
fn the_variants_that_withdraw_carry_the_name_they_looked_up() {
    let no_such = Error::NoSuchPackage {
        name: "shall-zzz-nope/nope".into(),
        message: "the repo has no published release".into(),
    };
    assert!(says_a_name_is_absent(&boxed(no_such.clone())));
    assert_eq!(no_such.absent_name(), Some("shall-zzz-nope/nope"));
    assert_eq!(
        backend_absent_name(&boxed(no_such)),
        Some("shall-zzz-nope/nope")
    );

    let unresolvable = Error::Unresolvable {
        name: "shall-no-such-pkg-zzz".into(),
        message: "no backend claims it".into(),
    };
    assert!(says_a_name_is_absent(&boxed(unresolvable.clone())));
    assert_eq!(unresolvable.absent_name(), Some("shall-no-such-pkg-zzz"));
    // A spawned manager's failure is the one that does *not* know which name, which is
    // why the edits are consulted for that case and only that case.
    assert_eq!(
        backend_absent_name(&boxed(Error::command_failed_absent("`npm` failed: 404"))),
        None
    );
}
