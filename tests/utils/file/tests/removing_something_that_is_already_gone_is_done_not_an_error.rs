use super::*;

#[test]
fn removing_something_that_is_already_gone_is_done_not_an_error() {
    let dir = TempDir::new().unwrap();
    force_remove(&dir.path().join("never-existed")).unwrap();
}
