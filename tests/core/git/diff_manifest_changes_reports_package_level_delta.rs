use super::*;

#[test]
fn diff_manifest_changes_reports_package_level_delta() {
    if !git_or_skip() {
        return;
    }
    let tmp = tempdir().unwrap();
    let git = GitManager::new(tmp.path());
    git.init().unwrap();
    std::fs::create_dir_all(tmp.path().join("modules")).unwrap();

    std::fs::write(tmp.path().join("modules/dev.txt"), "apt:curl\napt:nano\n").unwrap();
    git.commit_all("base").unwrap();
    std::fs::write(
        tmp.path().join("modules/dev.txt"),
        "apt:curl\ncargo:ripgrep\n",
    )
    .unwrap();
    git.commit_all("swap nano for ripgrep").unwrap();

    let changes = git.diff_manifest_changes("HEAD~1", Some("HEAD")).unwrap();
    // Package-level delta: nano removed, ripgrep added, curl untouched (in neither).
    assert!(changes.contains(&"- apt:nano".to_string()), "{:?}", changes);
    assert!(
        changes.contains(&"+ cargo:ripgrep".to_string()),
        "{:?}",
        changes
    );
    assert!(
        !changes.iter().any(|c| c.contains("apt:curl")),
        "unchanged lines must not appear: {:?}",
        changes
    );
}
