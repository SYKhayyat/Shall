use super::*;

#[test]
fn pixi_list_handles_tree_and_flat() {
    let tree =
        "Global environments at /root/.pixi/envs:\n├── python: 3.11.0\n└── ripgrep: 14.0.0\n";
    let pkgs = pixi_list(tree, "pixi").expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "python");
    assert_eq!(pkgs[0].version.as_deref(), Some("3.11.0"));
    assert_eq!(pkgs[1].name, "ripgrep");
}
