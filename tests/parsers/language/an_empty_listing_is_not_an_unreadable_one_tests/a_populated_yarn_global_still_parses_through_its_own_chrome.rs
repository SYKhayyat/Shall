use super::*;

#[test]
fn a_populated_yarn_global_still_parses_through_its_own_chrome() {
    let out = "yarn global v1.22.22
info \"typescript@5.4.5\" has binaries:
   - tsc
Done in 0.67s.
";
    let pkgs = parse_installed("yarn", out).expect("parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "typescript");
    assert_eq!(pkgs[0].version.as_deref(), Some("5.4.5"));
}
