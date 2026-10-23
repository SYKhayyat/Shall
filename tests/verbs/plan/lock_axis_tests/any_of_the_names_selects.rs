use super::*;

/// One name out of several still selects.
#[test]
fn any_of_the_names_selects() {
    assert!(scoped_by("apt:curl", &names(&["jq", "curl", "fd"])));
    assert!(!scoped_by("apt:curl", &names(&["jq", "fd"])));
}
