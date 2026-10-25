use super::*;

#[test]
fn two_matching_blocks_that_agree_are_redundant_not_wrong() {
    let v = resolve(&[
        top("role", "desktop", 1),
        when("role", "travel", 5),
        when("role", "travel", 9),
    ])
    .unwrap();
    assert_eq!(v["role"], str_val("travel"));
}
