use super::*;

#[test]
fn brewfile_lists_only_brew_packages() {
    let pkgs = vec![p("brew", "ripgrep", Some("14.1")), p("apt", "curl", None)];
    let out = Format::Brew.render(&pkgs).unwrap().unwrap();
    assert!(out.contains("brew \"ripgrep\"\n"));
    assert!(!out.contains("curl"));
}
