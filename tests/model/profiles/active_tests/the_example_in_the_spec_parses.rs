use super::*;

#[test]
fn the_example_in_the_spec_parses() {
    assert_eq!(on(II6_EXAMPLE, "laptop"), ["Work", "Gaming", "Travel"]);
}
