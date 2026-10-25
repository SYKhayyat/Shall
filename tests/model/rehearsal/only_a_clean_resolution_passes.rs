use super::*;

/// Only 0 is a pass. A rehearsal that accepted a second code would be accepting whatever
/// that code happens to mean next time the exit table is touched.
#[test]
fn only_a_clean_resolution_passes() {
    assert_eq!(verdict(Some(0)), Verdict::Valid);
    for code in [1, 2, 3, 101] {
        assert_eq!(
            verdict(Some(code)),
            Verdict::Rejected(code),
            "code {}",
            code
        );
    }
    // Killed by a signal: no code, and no evidence the config is fine.
    assert_eq!(verdict(None), Verdict::Rejected(1));
}
