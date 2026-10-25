use super::*;

#[test]
fn nothing_failing_is_healthy() {
    assert_eq!(Outcome::of(vec![], Some("snap-1")), Outcome::Healthy);
    assert_eq!(Outcome::of(vec![], None), Outcome::Healthy);
}
