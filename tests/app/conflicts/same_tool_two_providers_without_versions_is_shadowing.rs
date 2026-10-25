use super::*;

#[test]
fn same_tool_two_providers_without_versions_is_shadowing() {
    let specs = vec![spec("apt", "docker.io", None), spec("snap", "docker", None)];
    let c = detect_conflicts(&specs);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].name, "docker");
    assert_eq!(c[0].kind, ConflictKind::MultipleProviders);
}
