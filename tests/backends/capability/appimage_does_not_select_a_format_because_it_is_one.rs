use super::*;

#[test]
fn appimage_does_not_select_a_format_because_it_is_one() {
    assert!(!selects_artifacts("appimage"));
}
