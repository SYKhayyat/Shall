use super::*;

/// `sync` compares what was declared against what `list` reports, so the name `list`
/// reports must be the name `go install` takes — the package path. Preferring the `mod`
/// line reported `golang.org/x/tools` for a binary declared as
/// `golang.org/x/tools/cmd/goimports`: never equal, so drift that never converges, and an
/// `upgrade` that reinstalls a module with no program in it.
#[test]
fn the_identity_is_the_package_path_not_the_module() {
    let (name, version) = parse_go_version_m(SUBPACKAGE).unwrap();
    assert_eq!(name, "golang.org/x/tools/cmd/goimports");
    assert_eq!(version.as_deref(), Some("v0.48.0"));
}
