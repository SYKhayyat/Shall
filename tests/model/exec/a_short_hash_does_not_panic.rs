use super::*;

/// A short hash must not panic the describer — the ledger stores 64 hex chars, but nothing
/// in the type says so.
#[test]
fn a_short_hash_does_not_panic() {
    assert!(Decision::Run.describe("abc").contains("abc"));
    assert!(Decision::Run.describe("").contains("will run"));
}
