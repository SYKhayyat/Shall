use super::*;

#[test]
fn a_typo_on_a_setting_is_refused_like_any_other_extra() {
    let err = p("setting:org.gnome.x/k@vale=dark").unwrap_err();
    assert!(err.what.contains("is not an option"), "{}", err);
}
