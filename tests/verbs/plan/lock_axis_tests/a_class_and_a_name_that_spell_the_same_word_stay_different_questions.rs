use super::*;

/// **`versions:apt` means every apt package, and `apt` still means the package called
/// `apt`.** Both readings are live at once — `apt:apt` is a real entry on every Debian
/// machine — which is the whole reason the class is a qualifier and not a bare word.
#[test]
fn a_class_and_a_name_that_spell_the_same_word_stay_different_questions() {
    let apt_class = LockSelection::parse("versions:apt", &[]).unwrap();
    assert!(apt_class.admits(LockKind::Versions, "apt:curl", None));
    assert!(apt_class.admits(LockKind::Versions, "apt:apt", None));
    assert!(!apt_class.admits(LockKind::Versions, "cargo:apt", None));

    // The name reading, unchanged: `apt` picks out `apt:apt` and `cargo:apt` by their tail
    // and leaves `apt:curl` alone. The opposite of what the class reading selects.
    assert!(scoped_by("apt:apt", &names(&["apt"])));
    assert!(scoped_by("cargo:apt", &names(&["apt"])));
    assert!(!scoped_by("apt:curl", &names(&["apt"])));
}
