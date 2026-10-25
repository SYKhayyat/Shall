use super::*;

#[test]
fn a_reference_ends_at_a_non_name_character_without_braces() {
    let v = resolve(&[top("role", "render", 1), top("path", "/etc/$role/conf", 2)]).unwrap();
    assert_eq!(v["path"], str_val("/etc/render/conf"));
}
