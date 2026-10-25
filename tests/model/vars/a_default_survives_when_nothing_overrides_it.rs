use super::*;

#[test]
fn a_default_survives_when_nothing_overrides_it() {
    let v = resolve(&[top("role", "desktop", 1)]).unwrap();
    assert_eq!(v["role"], str_val("desktop"));
}
