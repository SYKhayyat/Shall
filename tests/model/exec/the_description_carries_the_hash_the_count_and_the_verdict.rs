use super::*;

/// `plan` must print the hash, the count and the decision before any of it happens — the
/// exit condition's fourth clause.
#[test]
fn the_description_carries_the_hash_the_count_and_the_verdict() {
    let hash = "a1b2c3d4e5f60718293a4b5c6d7e8f90";
    assert!(Decision::Run.describe(hash).contains("a1b2c3d4e5f6"));
    assert!(Decision::Run.describe(hash).contains("will run"));

    let done = Decision::AlreadyRun {
        count: 2,
        ceiling: Ceiling::read(Some("3")),
    };
    let line = done.describe(hash);
    assert!(line.contains("already run 2 time(s)"), "{}", line);
    assert!(line.contains("ceiling 3"), "{}", line);

    let unapproved = Decision::NeedsApproval(Verdict::New).describe(hash);
    assert!(unapproved.contains("NOT APPROVED"), "{}", unapproved);
    assert!(unapproved.contains("shall lock"), "{}", unapproved);

    let stale = Decision::NeedsApproval(changed()).describe(hash);
    assert!(stale.contains("CHANGED"), "{}", stale);
}
