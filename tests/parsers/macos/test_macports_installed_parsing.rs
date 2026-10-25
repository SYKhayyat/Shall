use super::*;

#[test]
fn test_macports_installed_parsing() {
    let input = "The following ports are currently installed:\n  \
                 git @2.39.0_0+doc (active)\n  wget @1.21.3_0\n";
    let res = parse_macports_installed(input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "git");
    assert_eq!(res[0].version.as_deref(), Some("2.39.0_0"));
    assert_eq!(res[1].name, "wget");
    assert_eq!(res[1].version.as_deref(), Some("1.21.3_0"));
    // The header line must not become a package.
    assert!(res.iter().all(|p| p.name != "The"));
}
