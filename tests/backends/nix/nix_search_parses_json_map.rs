use super::*;

#[test]
fn nix_search_parses_json_map() {
    let out = r#"{
        "legacyPackages.x86_64-linux.ripgrep": {"pname":"ripgrep","version":"14.1.0","description":"fast grep"},
        "legacyPackages.x86_64-linux.bat": {"pname":"bat","version":"0.24.0","description":"cat clone"}
    }"#;
    let pkgs = parse_nix_search(out).unwrap();
    assert_eq!(pkgs.len(), 2);
    // HashMap order is nondeterministic; assert by membership.
    let rg = pkgs
        .iter()
        .find(|p| p.name == "ripgrep")
        .expect("ripgrep present");
    assert_eq!(rg.version.as_deref(), Some("14.1.0"));
    assert!(rg.properties.get("attr_path").unwrap().ends_with("ripgrep"));
    assert!(pkgs.iter().any(|p| p.name == "bat"));
}
