use super::*;

#[test]
fn removing_a_directory_takes_the_whole_tree() {
    let dir = TempDir::new().unwrap();
    let tree = dir.path().join("a").join("b");
    std::fs::create_dir_all(&tree).unwrap();
    std::fs::write(tree.join("c"), b"x").unwrap();
    force_remove(&dir.path().join("a")).unwrap();
    assert!(!dir.path().join("a").exists());
}
