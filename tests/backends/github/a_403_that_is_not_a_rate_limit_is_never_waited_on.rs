use super::*;

#[test]
fn a_403_that_is_not_a_rate_limit_is_never_waited_on() {
    // A bad token or a private repo: no header, and no amount of waiting changes it.
    assert_eq!(
        rate_limit_action(403, None, 1000, 30),
        RateLimit::NotLimited
    );
    // A header that is not a number is not a reset time either.
    assert_eq!(
        rate_limit_action(403, Some("soon"), 1000, 30),
        RateLimit::NotLimited
    );
    // And a limit that has already reset is over, whatever the header says.
    assert_eq!(
        rate_limit_action(403, Some("999"), 1000, 30),
        RateLimit::NotLimited
    );
}
