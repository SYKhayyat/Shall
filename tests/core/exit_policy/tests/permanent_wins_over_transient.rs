use super::*;

/// A manager that prints both must not loop on the half that cannot succeed.
#[test]
fn permanent_wins_over_transient() {
    // A genuinely *permanent* marker, which is what this test is named for. It used to
    // assert the same thing with apt's `Unable to locate package` — an ABSENT marker — and
    // so quietly also pinned absent-over-transient, which is the pairing that made a
    // dropped connection delete declarations. That half now has its own test below.
    let p = cargo();
    let both = b"spurious network error\nerror: there are no binaries" as &[u8];
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(b"", both)),
        Retryability::Permanent
    );
}
