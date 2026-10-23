use super::*;

/// The tool's own output, not a hand-written imitation of it: the invented fixture had two
/// rows, no nested child and a banner in a wording pixi no longer uses, so it passed while
/// the parser reported `exposes` as an installed package (GRADER §3.3).
#[test]
fn pixi_list_reads_the_tools_own_output() {
    const LIST: &str = include_str!("../../../../tests/fixtures/pixi/list-one-tool.txt");
    let pkgs = pixi_list(LIST, "pixi").expect("this fixture parses");
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["ripgrep"],
        "an `exposes:` child is not a package"
    );
    assert_eq!(pkgs[0].version.as_deref(), Some("15.2.0"));
}
