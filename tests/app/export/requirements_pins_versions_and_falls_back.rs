use super::*;

#[test]
fn requirements_pins_versions_and_falls_back() {
    let pkgs = vec![
        p("pip", "requests", Some("2.31.0")),
        p("pipx", "black", None),
        p("npm", "left-pad", Some("1.3.0")),
    ];
    let out = Format::Pip.render(&pkgs).unwrap().unwrap();
    assert!(out.contains("requests==2.31.0\n"));
    assert!(out.contains("black\n"));
    assert!(!out.contains("left-pad"));
}
