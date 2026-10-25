use super::*;

#[test]
fn version_mismatch_across_backends_is_flagged() {
    // apt:nodejs@18 vs nix:node@20 — same canonical tool, different versions.
    let specs = vec![
        spec("apt", "nodejs", Some("18")),
        spec("nix", "node", Some("20")),
    ];
    let c = detect_conflicts(&specs);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].name, "node");
    assert_eq!(c[0].kind, ConflictKind::VersionMismatch);
    assert_eq!(c[0].providers.len(), 2);
}
