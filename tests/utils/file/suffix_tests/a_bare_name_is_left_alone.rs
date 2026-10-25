use super::*;

#[test]
fn a_bare_name_is_left_alone() {
    assert_eq!(strip_archive_suffixes("jq"), "jq");
    assert_eq!(strip_archive_suffixes("socket.io"), "socket.io");
}
