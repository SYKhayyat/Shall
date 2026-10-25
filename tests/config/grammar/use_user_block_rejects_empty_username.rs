use super::*;

#[test]
fn use_user_block_rejects_empty_username() {
    let err = doc("use user: {\n  link:./vimrc@target=~/vimrc\n}\n").unwrap_err();
    assert!(err.to_string().contains("no username"), "{}", err);
}
