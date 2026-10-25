use super::*;

#[test]
fn the_ceiling_is_the_only_thing_that_decides_between_them() {
    // The same response, one machine willing to wait an hour and one not.
    assert_eq!(
        rate_limit_action(403, Some("4600"), 1000, 7200),
        RateLimit::WaitThenRetry(3601)
    );
}
