use super::*;

/// A plugin source that carries no signature never grows one, so retrying is time spent
/// reaching the same refusal. Measured against helm v4.2.3 on 2026-07-28.
#[test]
fn cabal_reads_an_unverifiable_hackage_root_as_the_index_and_not_as_the_package() {
    let p = cabal();
    let tuf = ExitPolicy::haystack(
        b"<repo>/root.json does not have enough signatures signed with the appropriate keys",
        b"",
    );
    assert_eq!(p.retryability(&tuf), Retryability::Transient);
    // The whole reason this is not `Permanent`: withdrawing here deletes a declaration
    // whose package is real and whose machine only needs its trust anchor refreshed.
    assert!(!p.names_an_absent_package(&tuf));
}
