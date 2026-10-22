use super::*;

#[test]
fn a_setting_with_no_value_declares_nothing() {
    let err = p("setting:org.gnome.x/color-scheme").unwrap_err();
    assert!(err.what.contains("no value"), "{}", err);
}
