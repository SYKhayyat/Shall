use super::*;

#[test]
fn empty_when_no_package_applies() {
    let pkgs = vec![p("cargo", "ripgrep", Some("14.1"))];
    assert!(matches!(Format::Brew.render(&pkgs), Ok(None)));
    assert!(matches!(Format::Apt.render(&pkgs), Ok(None)));
}
