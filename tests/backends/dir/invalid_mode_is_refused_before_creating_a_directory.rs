use super::*;

#[tokio::test]
async fn invalid_mode_is_refused_before_creating_a_directory() {
    let root = tempdir().unwrap();
    let path = root.path().join("invalid");
    let error = installer()
        .install(&[spec(path.to_str().unwrap(), &[("mode", "10000")])], false)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("invalid mode"));
    assert!(!path.exists());
}
