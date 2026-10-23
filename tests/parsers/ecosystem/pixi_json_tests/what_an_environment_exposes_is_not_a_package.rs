use super::*;

/// `exposes` is what the environment puts on PATH, not a second tool. The tree parser
/// reported it as a package once; the JSON parser must not find a new way to.
#[test]
fn what_an_environment_exposes_is_not_a_package() {
    let pkgs = pixi_list_json(REAL, "pixi").expect("this fixture parses");
    assert!(!pkgs.iter().any(|p| p.name == "rg" || p.name == "exposed"));
}
