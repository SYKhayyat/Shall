use super::*;

#[test]
fn matching_versions_across_backends_is_shadowing_not_mismatch() {
    let specs = vec![
        spec("apt", "go", Some("1.22")),
        spec("nix", "golang", Some("1.22")),
    ];
    let c = detect_conflicts(&specs);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].kind, ConflictKind::MultipleProviders);
}
