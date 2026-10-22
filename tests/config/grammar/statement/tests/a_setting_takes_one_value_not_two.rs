use super::*;

#[test]
fn a_setting_takes_one_value_not_two() {
    let err = p("setting:org.gnome.x/k@value=a,value=b").unwrap_err();
    assert!(err.what.contains("one value"), "{}", err);
}
