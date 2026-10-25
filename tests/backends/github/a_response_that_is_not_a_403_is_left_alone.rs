use super::*;

#[test]
fn a_response_that_is_not_a_403_is_left_alone() {
    for status in [200, 404, 429, 500] {
        assert_eq!(
            rate_limit_action(status, Some("99999"), 1000, 30),
            RateLimit::NotLimited,
            "status {} was treated as a rate limit",
            status
        );
    }
}
