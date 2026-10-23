use super::*;

#[test]
fn an_environment_is_one_package_at_its_own_version() {
    let pkgs = pixi_list_json(REAL, "pixi").expect("this fixture parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "ripgrep");
    assert_eq!(pkgs[0].version.as_deref(), Some("15.2.0"));
    assert_eq!(pkgs[0].backend, "pixi");
}
