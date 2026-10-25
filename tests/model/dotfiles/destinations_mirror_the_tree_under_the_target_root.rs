use super::*;

#[test]
fn destinations_mirror_the_tree_under_the_target_root() {
    let tree = tree_with(&[".config/nvim/init.lua"]);
    let home = Path::new("/home/me");
    let p = plan(tree.path(), home, &nothing_exists, &nothing_owned).unwrap();
    assert_eq!(
        p.placements[0].destination,
        home.join(".config").join("nvim").join("init.lua")
    );
}
