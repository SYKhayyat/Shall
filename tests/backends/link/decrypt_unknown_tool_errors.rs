use super::*;

#[tokio::test]
async fn decrypt_unknown_tool_errors() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("s.enc");
    tokio::fs::write(&source, "x").await.unwrap();
    let target = dir.path().join("out");
    let inst = installer(); // real executor
    let r = inst
        .install(&[decrypt_spec(&source, &target, "rot13")], false)
        .await;
    assert!(r.is_err(), "an unknown decrypt tool must be rejected");
}
