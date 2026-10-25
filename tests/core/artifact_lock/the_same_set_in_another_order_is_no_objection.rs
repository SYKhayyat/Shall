use super::*;

#[test]
fn the_same_set_in_another_order_is_no_objection() {
    // A release that reorders its assets did not change what is installed.
    let locked = [lock("a.tar.gz", Some("a1")), lock("b.tar.gz", Some("b2"))];
    assert!(verify_set(
        &locked,
        &[
            ("b.tar.gz".into(), Some("b2".into())),
            ("a.tar.gz".into(), Some("a1".into())),
        ]
    )
    .is_none());
}
