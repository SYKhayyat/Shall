use super::*;

#[test]
fn a_directory_that_cannot_be_created_says_which_one() {
    // `Access is denied. (os error 5)` with no path is the failure this replaces: a sync
    // touches a dozen directories and the message named none of them.
    let dir = TempDir::new().unwrap();
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").unwrap();
    let err = ensure_dir(&blocker.join("under")).unwrap_err().to_string();
    assert!(
        err.contains("blocker"),
        "the error does not name the directory: {err}"
    );
}
