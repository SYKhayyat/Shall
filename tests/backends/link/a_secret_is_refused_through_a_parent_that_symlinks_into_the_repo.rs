//! **The destination is refused for where it LANDS, not for where it is written.** A first
//! install has no file at the target to canonicalize, which is why the old guard compared
//! strings — and a string comparison cannot see that a parent directory is a symlink. This is
//! the whole bypass: a path that reads as outside the repo, whose parent points into it, and a
//! plaintext in git history at the end of it.

use super::*;

#[cfg(unix)]
#[test]
fn a_secret_is_refused_through_a_parent_that_symlinks_into_the_repo() {
    let repo = tempdir().unwrap();
    let elsewhere = tempdir().unwrap();
    let config = Config {
        config_root: repo.path().to_path_buf(),
        ..Default::default()
    };

    // `elsewhere/tokens` is outside the repo and points into it, which is what a well-meaning
    // symlink in somebody's dotfiles looks like. The directory it points at exists: a dangling
    // one would make the write fail with ENOENT instead of landing in the repo, which is the
    // only case where the answer this guard gives does not matter.
    std::fs::create_dir_all(repo.path().join("secrets")).expect("the directory inside the repo");
    std::os::unix::fs::symlink(repo.path().join("secrets"), elsewhere.path().join("tokens"))
        .expect("the symlink");

    // The file does not exist: this is a first install, and it is the case the old guard fell
    // back to a string comparison for.
    let target = elsewhere.path().join("tokens").join("token");
    assert!(!target.exists());
    assert!(
        !target.starts_with(repo.path()),
        "the test is not testing anything: the path reads as outside the repo"
    );

    let err = refuse_target_in_repo(&config, &target)
        .expect_err("the plaintext lands inside the repo whatever the path says")
        .to_string();
    assert!(err.contains("config repo"), "{err}");

    // And a symlinked parent that points *out* of the repo is the other direction: a path
    // written inside the repo that lands outside it is not a secret in git, so it is not
    // refused. Fail-closed would be defensible here; fail-open on a decryption target is not
    // the failure this guard exists to prevent.
    let outer = tempdir().unwrap();
    let inside_repo_but_out = repo.path().join("linked-out");
    std::os::unix::fs::symlink(outer.path(), &inside_repo_but_out).expect("the symlink");
    refuse_target_in_repo(&config, &inside_repo_but_out).expect("lands outside the repo");
}
