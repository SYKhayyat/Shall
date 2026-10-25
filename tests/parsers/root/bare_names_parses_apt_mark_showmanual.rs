use super::*;

#[test]
fn bare_names_parses_apt_mark_showmanual() {
    // `apt-mark showmanual` prints bare names, no versions â€” which is why the normal
    // apt list parser (which splits "name version") silently returned nothing.
    let pkgs = parse_bare_names("apt\nbase-files\njq\n", "apt").expect("this fixture parses");
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["apt", "base-files", "jq"]);
    assert_eq!(pkgs[0].backend, "apt");
}
