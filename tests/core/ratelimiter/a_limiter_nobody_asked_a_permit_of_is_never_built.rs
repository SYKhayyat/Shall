use super::*;

/// `AU3`: a backend's `new` runs for every subcommand, including the ones that touch no
/// network at all. `github`'s cost 200 ms building a clock for a budget the run never spent.
#[test]
fn a_limiter_nobody_asked_a_permit_of_is_never_built() {
    let l = RateLimiter::new(60, "unused");
    assert!(!l.is_engaged(), "constructing the limiter built its issuer");
    assert_eq!(l.description(), "unused");
}
