use super::*;

#[test]
fn a_hostname_cannot_write_outside_the_locks_directory() {
    let path = BareLock::path_for(Path::new("locks"), "../../etc/passwd");
    assert_eq!(path.parent(), Some(Path::new("locks")));
}
