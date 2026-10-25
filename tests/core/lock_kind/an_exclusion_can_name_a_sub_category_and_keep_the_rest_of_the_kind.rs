use super::*;

/// **The exclusion that has no spelling as a flag, and the reason the scope is a word.**
/// Everything except one manager's pins: the kind is still wanted, one sub-category is not.
#[test]
fn an_exclusion_can_name_a_sub_category_and_keep_the_rest_of_the_kind() {
    let s = LockSelection::parse("everything", &words(&["versions:cargo"])).unwrap();
    assert!(s.includes(LockKind::Versions), "the kind is still wanted");
    assert!(s.admits(LockKind::Versions, "apt:curl", None));
    assert!(
        !s.admits(LockKind::Versions, "cargo:ripgrep", None),
        "cargo's pins were excluded and are still being taken"
    );
    // Untouched kinds are untouched.
    assert!(s.admits(LockKind::Exec, "exec:./setup.sh", None));
    assert!(s.admits(LockKind::Hooks, "after_install:nginx", None));
}
