use super::*;

#[test]
fn checkout_files_restores_manifest_without_new_commit() {
    if !git_or_skip() {
        return;
    }
    let tmp = tempdir().unwrap();
    let git = GitManager::new(tmp.path());
    git.init().unwrap();
    let manifest = tmp.path().join("local.txt");

    std::fs::write(&manifest, "apt:curl\n").unwrap();
    let c1 = git.commit_all("v1").unwrap().unwrap();
    std::fs::write(&manifest, "apt:curl\napt:htop\n").unwrap();
    git.commit_all("v2").unwrap();

    // Roll the manifest back to v1's content; installed packages are irrelevant here.
    git.checkout_files(&c1).unwrap();
    // Normalize line endings: git on Windows may apply autocrlf on checkout. Shall reads
    // manifests via `.lines()`, which tolerates CRLF, so this is cosmetic.
    let restored = std::fs::read_to_string(&manifest)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(
        restored, "apt:curl\n",
        "working tree restored to the v1 manifest"
    );
}
