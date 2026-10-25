use super::*;

#[test]
fn two_matching_blocks_that_disagree_name_both_lines() {
    let err = resolve(&[
        top("role", "desktop", 1),
        when("role", "travel", 5),
        when("role", "workstation", 9),
    ])
    .unwrap_err();
    assert!(err.what.contains("travel"), "{}", err);
    assert!(err.what.contains("workstation"), "{}", err);
    assert!(err.what.contains("vars:5"), "names the other line: {}", err);
}
