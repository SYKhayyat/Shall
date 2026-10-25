use super::*;

/// II.12 outranks the count: a script that changed under you is a refusal, not a skip.
/// Reporting "already run" here would hide the exact edit the ledger exists to catch.
#[test]
fn approval_is_checked_before_the_count() {
    let d = Decision::of(&changed(), 1, Ceiling::read(None));
    assert!(matches!(d, Decision::NeedsApproval(_)));
    assert!(!d.will_run());

    let d = Decision::of(&Verdict::New, 0, Ceiling::read(None));
    assert!(matches!(d, Decision::NeedsApproval(Verdict::New)));
}
