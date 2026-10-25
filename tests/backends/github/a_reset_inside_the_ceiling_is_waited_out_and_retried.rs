use super::*;

#[test]
fn a_reset_inside_the_ceiling_is_waited_out_and_retried() {
    assert_eq!(
        rate_limit_action(403, Some("1000"), 980, 30),
        RateLimit::WaitThenRetry(21)
    );
}
