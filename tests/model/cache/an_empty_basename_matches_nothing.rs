use super::*;

#[test]
fn an_empty_basename_matches_nothing() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("x"), b"x").unwrap();
    assert!(find_cached("", std::slice::from_ref(&dir.path().to_path_buf())).is_empty());
}
