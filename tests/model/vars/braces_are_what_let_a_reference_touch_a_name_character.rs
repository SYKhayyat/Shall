use super::*;

#[test]
fn braces_are_what_let_a_reference_touch_a_name_character() {
    let v = resolve(&[top("role", "render", 1), top("tier", "$role_x", 2)]);
    assert!(
        v.is_err(),
        "`$role_x` must not silently resolve to `render_x`"
    );
}
