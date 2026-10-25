use super::*;

#[test]
fn a_reset_past_the_ceiling_is_refused_with_the_wait_it_would_have_taken() {
    assert_eq!(
        rate_limit_action(403, Some("4600"), 1000, 30),
        RateLimit::TooLong(3601)
    );
}
