use super::*;

/// The JSON form must report exactly what the text form does — a difference between them
/// is a machine that changes shape depending on which pixi is installed.
#[test]
fn the_json_and_the_tree_agree_about_the_same_machine() {
    let tree = "Global environments as specified in '/tmp/pixi-global.toml'\n\
                └── ripgrep: 15.2.0 \n    └─ exposes: rg\n";
    let from_tree = pixi_list(tree, "pixi").expect("this fixture parses");
    let from_json = pixi_list_json(REAL, "pixi").expect("this fixture parses");
    assert_eq!(
        from_tree
            .iter()
            .map(|p| (&p.name, &p.version))
            .collect::<Vec<_>>(),
        from_json
            .iter()
            .map(|p| (&p.name, &p.version))
            .collect::<Vec<_>>(),
    );
}
