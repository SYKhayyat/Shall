use super::*;

/// **`kind:qualifier` selects part of a kind.** One manager's pins, one hook across every
/// package, one event wherever it is declared — and in each case the neighbours are not
/// selected, which is the half that proves the narrowing happened.
#[test]
fn a_qualifier_selects_one_sub_category_and_not_its_neighbours() {
    let apt = sel("versions:apt");
    assert!(apt.admits(LockKind::Versions, "apt:curl", None));
    assert!(!apt.admits(LockKind::Versions, "cargo:ripgrep", None));

    let after = sel("hooks:after_install");
    assert!(after.admits(LockKind::Hooks, "after_install:nginx", None));
    assert!(!after.admits(LockKind::Hooks, "before_install:nginx", None));

    // A backends entry is keyed by the bare name, so its manager comes from the value.
    let cargo = sel("backends:cargo");
    assert!(cargo.admits(LockKind::Backends, "ripgrep", Some("cargo")));
    assert!(!cargo.admits(LockKind::Backends, "curl", Some("apt")));
}
