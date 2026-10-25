use super::*;

/// U22: files, not directories. A tree containing `.config/nvim/init.lua` places that one
/// file — it does not place `.config/nvim`, which would take the directory hostage.
#[test]
fn the_tree_places_files_not_directories() {
    let tree = tree_with(&[".config/nvim/init.lua", ".bashrc"]);
    let home = Path::new("/home/me");
    let p = plan(tree.path(), home, &nothing_exists, &nothing_owned).unwrap();

    let rels: Vec<String> = p
        .placements
        .iter()
        .map(|x| x.relative.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(rels, vec![".bashrc", ".config/nvim/init.lua"]);
    // The directory itself is never a placement.
    assert!(!rels.iter().any(|r| r == ".config/nvim"));
}
