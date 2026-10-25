use super::*;

/// npm prints a prefix, and where the executables sit under it is a platform rule. Both
/// readings count, so being wrong about the rule cannot warn a user whose machine is fine.
#[test]
fn npms_prefix_offers_both_readings() {
    let prefix = PathBuf::from(SANDBOX).join("npm-prefix");
    let candidates = npm_prefix_candidates("npm", prefix.clone());
    assert!(
        candidates.contains(&prefix),
        "the prefix itself is a candidate: {candidates:?}"
    );
    assert!(
        candidates.contains(&prefix.join("bin")),
        "`<prefix>/bin` is a candidate: {candidates:?}"
    );
    // Every other manager answers with the bin directory itself; there is nothing to widen.
    assert_eq!(npm_prefix_candidates("yarn", prefix.clone()), vec![prefix]);
}
