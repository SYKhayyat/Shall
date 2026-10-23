use super::*;

/// A dependency that is not the environment's own name is a dependency, not something the
/// user asked for. Adopting those would write a dependency graph into a manifest.
#[test]
fn a_pulled_in_dependency_is_not_reported_as_installed() {
    let json = r#"[{"name":"ripgrep","dependencies":[
        {"name":"ripgrep","version":"15.2.0"},
        {"name":"libgcc","version":"14.1"}]}]"#;
    let pkgs = pixi_list_json(json, "pixi").expect("this fixture parses");
    assert_eq!(pkgs.len(), 1, "got {:?}", pkgs);
    assert_eq!(pkgs[0].name, "ripgrep");
}
