use super::*;

#[test]
fn a_tool_is_its_package_id_and_version() {
    let pkgs = parse_dotnet_list_json(REAL).expect("this fixture parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "dotnetsay");
    assert_eq!(pkgs[0].version.as_deref(), Some("3.0.3"));
    assert_eq!(pkgs[0].backend, "dotnet");
}
