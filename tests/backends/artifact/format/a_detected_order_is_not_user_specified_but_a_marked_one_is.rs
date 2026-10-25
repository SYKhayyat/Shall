use super::*;

#[test]
fn a_detected_order_is_not_user_specified_but_a_marked_one_is() {
    assert!(!FormatOrder::detected_default("linux", Some("debian")).is_user_specified());
    assert!(FormatOrder::new(vec![Format::Deb])
        .as_user_specified()
        .is_user_specified());
}
