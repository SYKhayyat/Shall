use super::*;

#[test]
fn use_user_block_rejects_unsupported_statements() {
    let err = doc("use user:alice {\n  apt:curl\n}\n").unwrap_err();
    assert!(err.to_string().contains("does not support"), "{}", err);
    assert_eq!(err.origin.line, 2);
}
