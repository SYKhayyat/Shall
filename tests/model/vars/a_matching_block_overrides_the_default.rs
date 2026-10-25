use super::*;

#[test]
fn a_matching_block_overrides_the_default() {
    let v = resolve(&[top("role", "desktop", 1), when("role", "travel", 5)]).unwrap();
    assert_eq!(v["role"], str_val("travel"));
}
