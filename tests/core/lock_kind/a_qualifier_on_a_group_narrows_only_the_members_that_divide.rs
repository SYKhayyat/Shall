use super::*;

/// A qualifier on a group narrows the members that divide and drops the ones that do not:
/// `packages:apt` is apt's pins and apt's resolutions, and claims nothing about `exec`.
#[test]
fn a_qualifier_on_a_group_narrows_only_the_members_that_divide() {
    let apt = sel("packages:apt");
    assert_eq!(kinds_of(&apt), PACKAGES.to_vec());
    assert!(apt.admits(LockKind::Versions, "apt:curl", None));
    assert!(!apt.admits(LockKind::Versions, "cargo:ripgrep", None));

    // `everything:apt` is the same rule over the wider group: every kind that divides is
    // narrowed to `apt`, and the five flat kinds drop out. It reaches `hooks` and `events`
    // too — a hook literally named `apt` — which is odd to read and is the price of one
    // rule applied uniformly. The alternative, a group that quietly means something
    // narrower than its own members, is worse: `packages:apt` and `everything:apt` would
    // then subset differently for no reason a user could state.
    let wide = sel("everything:apt");
    assert_eq!(
        kinds_of(&wide),
        vec![
            LockKind::Versions,
            LockKind::Backends,
            LockKind::Hooks,
            LockKind::Events
        ]
    );
    assert!(
        !wide.includes(LockKind::Exec),
        "a flat kind is not narrowed, it is dropped"
    );
}
