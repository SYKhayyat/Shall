use super::*;

#[tokio::test]
async fn a_body_under_the_ceiling_is_written_whole() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("artifact");
    let staged = stage_capped_to(undeclared("0123456789"), &dest, "an artifact", Some(1024))
        .await
        .expect("ten bytes under a kilobyte ceiling");
    assert_eq!(staged.bytes(), 10);
    assert_eq!(
        std::fs::read(staged.path()).expect("the file was staged"),
        b"0123456789"
    );
    staged
        .commit()
        .await
        .expect("the commit renames the staged file");
    assert_eq!(
        std::fs::read(&dest).expect("the file was written"),
        b"0123456789"
    );
}
