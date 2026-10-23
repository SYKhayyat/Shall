use super::*;

#[test]
fn asdf_list_keeps_plugins_skips_versions() {
    let out = "nodejs\n  18.0.0\n  20.0.0\npython\n  3.11.0\n";
    let pkgs = asdf_list(out, "asdf").expect("this fixture parses");
    assert_eq!(
        pkgs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        vec!["nodejs", "python"]
    );
}
