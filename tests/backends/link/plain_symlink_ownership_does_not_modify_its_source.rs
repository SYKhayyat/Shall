use super::*;

#[cfg(unix)]
#[tokio::test]
async fn plain_symlink_ownership_does_not_modify_its_source() {
    use std::os::unix::fs::MetadataExt;
    let dir = tempdir().unwrap();
    let source = dir.path().join("source");
    let target = dir.path().join("target");
    std::fs::write(&source, "source").unwrap();
    let source_before = std::fs::symlink_metadata(&source).unwrap();
    let account = shall::core::account::Account::current().unwrap();
    let mut options = shall::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
    options.set("user", account.name());
    let spec = PackageSpec {
        name: source.to_string_lossy().to_string(),
        backend: "link".into(),
        options,
        requires: vec![],
        present: true,
    };
    installer().install(&[spec], false).await.unwrap();
    let source_metadata = std::fs::symlink_metadata(&source).unwrap();
    assert_eq!(source_metadata.uid(), source_before.uid());
    assert_eq!(source_metadata.gid(), source_before.gid());
}
