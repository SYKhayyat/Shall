use super::*;

#[test]
fn a_populated_pipx_still_parses() {
    // The other half: the JSON short-circuit must not swallow a real answer.
    let out = r#"{"pipx_spec_version":"0.1","venvs":{"black":{"metadata":{"main_package":{"package":"black","package_version":"24.1.0"}}}}}"#;
    let pkgs = parse_installed("pipx", out).expect("parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "black");
    assert_eq!(pkgs[0].version.as_deref(), Some("24.1.0"));
}
