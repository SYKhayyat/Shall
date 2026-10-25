use super::*;

#[tokio::test]
async fn a_home_relative_dir_is_still_resolved() {
    let account = Account::current().unwrap();
    let installer = installer();
    let declaration = spec(
        &format!("~/.local/state/shall-abs-test-{}", std::process::id()),
        &[],
    );
    installer.install(&[declaration], false).await.unwrap();
    let resolved = account
        .home()
        .join(".local/state")
        .join(format!("shall-abs-test-{}", std::process::id()));
    assert!(resolved.is_dir(), "{}", resolved.display());
    let _ = std::fs::remove_dir(&resolved);
}
