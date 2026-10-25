use super::*;

#[test]
fn derived_values_resolve_in_dependency_order_not_file_order() {
    let v = resolve(&[
        top("tier", "${role}-heavy", 1),
        top("role", "render", 2),
        top("label", "${tier}!", 3),
    ])
    .unwrap();
    assert_eq!(v["label"], str_val("render-heavy!"));
}
