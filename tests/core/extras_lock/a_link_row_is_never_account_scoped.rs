use super::*;

#[test]
fn a_link_row_is_never_account_scoped() {
    let key = ExtraKey::link(Path::new("/home/alice/.vimrc"));
    assert_eq!(key.to_string(), "link:/home/alice/.vimrc");
    assert_eq!(key.account_suffix(), None);
    let refused = ExtraKey::with_user(ResourceKind::Link, "/home/alice/.vimrc", Some("alice"))
        .expect_err("a link row is a path, not an account");
    assert!(
        refused.to_string().contains("not keyed by account"),
        "{refused}"
    );
}
