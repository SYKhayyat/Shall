use super::*;

#[test]
fn an_export_yields_the_installed_apps() {
    let pkgs = parse_scoop_export(EXPORT).expect("this fixture parses");
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["7zip", "fd"]);
    assert_eq!(pkgs[0].version.as_deref(), Some("26.00"));
    assert!(pkgs.iter().all(|p| p.backend == "scoop"));
}
