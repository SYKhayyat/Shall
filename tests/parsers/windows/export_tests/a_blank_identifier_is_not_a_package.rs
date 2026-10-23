use super::*;

/// A blank identifier is not a package name, and the validator would refuse it later with
/// a worse message than "nothing was adopted".
#[test]
fn a_blank_identifier_is_not_a_package() {
    let pkgs = parse_winget_export(
        r#"{"Sources":[{"Packages":[{"PackageIdentifier":"  "},{"PackageIdentifier":"jq.jq"}]}]}"#,
    )
    .expect("this fixture parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "jq.jq");
}
