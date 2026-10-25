use super::*;

/// `["*"]` is every manager; a named list is those managers and no others. Both directions,
/// because a `pins` that answered `true` unconditionally would satisfy the first alone.
#[test]
fn a_named_manager_list_pins_those_managers_and_no_others() {
    let every = LockSettings::default();
    for backend in ["apt", "cargo", "brew", "a-backend-invented-tomorrow"] {
        assert!(every.pins(backend), "`*` must admit {backend}");
    }

    let narrowed = LockSettings {
        versions: vec!["apt".into(), "cargo".into()],
        ..LockSettings::default()
    };
    assert!(narrowed.pins("apt"));
    assert!(narrowed.pins("cargo"));
    assert!(!narrowed.pins("brew"), "brew was not named");
    assert!(!narrowed.pins("ap"), "a prefix is not a manager");

    // An empty list pins nothing, which is a real thing to want — record no versions at all
    // while still using `lock scripts`. It must not read as "unset, so everything".
    let none = LockSettings {
        versions: vec![],
        ..LockSettings::default()
    };
    assert!(!none.pins("apt"));
}
