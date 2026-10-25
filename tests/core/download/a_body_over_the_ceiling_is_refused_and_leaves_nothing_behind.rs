use super::*;

#[tokio::test]
async fn a_body_over_the_ceiling_is_refused_and_leaves_nothing_behind() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("artifact");
    let err = stage_capped_to(undeclared("0123456789"), &dest, "an artifact", Some(4))
        .await
        .expect_err("ten bytes under a four-byte ceiling");
    let message = err.to_string();
    assert!(message.contains("ceiling"), "{message}");
    assert!(
        !dest.exists(),
        "the partial file survived, and a later run would read it as a complete download"
    );
}
