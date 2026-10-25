use super::*;

/// The repository's own metadata is not a dotfile. Mirroring `.git` into `$HOME` would put
/// a second repository there.
#[test]
fn the_trees_own_git_metadata_is_never_placed() {
    let tree = tree_with(&[".git/config", ".gitignore", ".vimrc"]);
    let p = plan(
        tree.path(),
        Path::new("/home/me"),
        &nothing_exists,
        &nothing_owned,
    )
    .unwrap();
    let rels: Vec<String> = p
        .placements
        .iter()
        .map(|x| x.relative.to_string_lossy().into())
        .collect();
    assert_eq!(rels, vec![".vimrc"]);
}
