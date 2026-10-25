use super::*;

#[test]
fn commit_row_shows_hash_and_subject() {
    let row = commit_row(&cv("a1b2c3d", "add ripgrep", &["+ cargo:rg"]));
    assert!(row.contains("a1b2c3d"));
    assert!(row.contains("add ripgrep"));
}
