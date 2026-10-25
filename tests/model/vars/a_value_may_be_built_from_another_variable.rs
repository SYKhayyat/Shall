use super::*;

#[test]
fn a_value_may_be_built_from_another_variable() {
    let v = resolve(&[top("role", "render", 1), top("tier", "${role}-heavy", 2)]).unwrap();
    assert_eq!(v["tier"], str_val("render-heavy"));
}
