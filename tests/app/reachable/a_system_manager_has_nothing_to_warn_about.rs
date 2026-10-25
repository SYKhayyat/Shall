use super::*;

/// And the other half: a system manager must NOT warn. `/usr/bin` is on every PATH, so a
/// warning about it is noise on every install, and noise is how a real warning gets
/// ignored.
#[tokio::test]
async fn a_system_manager_has_nothing_to_warn_about() {
    let (exec, cfg) = mocked(&[]);
    for be in [
        "apt", "dnf", "pacman", "apk", "brew", "scoop", "winget", "choco",
    ] {
        assert!(
            user_bin_dirs(be, &cfg, &exec).await.is_empty(),
            "{be} installs into a system directory; warning about its PATH is noise"
        );
        assert!(unreachable_warning(be, &cfg, &exec).await.is_none());
    }
}
