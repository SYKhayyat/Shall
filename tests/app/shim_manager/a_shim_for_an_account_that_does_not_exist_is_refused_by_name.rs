use super::*;

#[tokio::test]
async fn a_shim_for_an_account_that_does_not_exist_is_refused_by_name() {
    let tmp = tempdir().unwrap();
    let err = shim_target(&tmp.path().join("bin"), Some("shall-no-such-account-71"))
        .expect_err("an account that cannot be resolved must be refused");
    let err = err.to_string();
    assert!(err.contains("shall-no-such-account-71"), "{err}");
}
