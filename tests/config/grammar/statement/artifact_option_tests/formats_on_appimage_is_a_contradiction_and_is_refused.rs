use super::*;

#[test]
fn formats_on_appimage_is_a_contradiction_and_is_refused() {
    assert!(p("appimage:foo@formats=deb").is_err());
}
