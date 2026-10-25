use super::*;

#[test]
fn a_client_that_would_follow_a_downgrade_is_never_handed_to_one_that_refuses() {
    let _strict = client("shall-pool-b", false, 15).unwrap();
    let _loose = client("shall-pool-b", true, 15).unwrap();
    assert_eq!(
        mine("shall-pool-b"),
        2,
        "the two redirect policies collapsed into one client — SEC2 would be enforced by \
         whichever caller happened to ask first"
    );
}
