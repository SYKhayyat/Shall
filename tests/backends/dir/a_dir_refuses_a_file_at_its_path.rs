use super::*;

#[tokio::test]
async fn a_dir_refuses_a_file_at_its_path() {
    let root = tempdir().unwrap();
    let path = root.path().join("not-a-directory");
    std::fs::write(&path, "file").unwrap();
    let error = installer()
        .install(&[spec(path.to_str().unwrap(), &[])], false)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("not a directory"));
    assert_eq!(std::fs::read(&path).unwrap(), b"file");
}
