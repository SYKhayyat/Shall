use super::*;

/// No checks declared, no opinion — the refusal must not fire on machines that never asked
/// for health checks, which is most of them.
#[test]
fn no_checks_means_no_refusal_either_way() {
    assert_eq!(refusal_if_unrevertable(&[], false, false), None);
    assert_eq!(refusal_if_unrevertable(&[], true, false), None);
}
