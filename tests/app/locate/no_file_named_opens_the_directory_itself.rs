use super::*;

#[test]
fn no_file_named_opens_the_directory_itself() {
    let dir = TempDir::new().unwrap();
    let target = resolve_target(dir.path(), None).unwrap();
    assert_eq!(target, dir.path());
}
