use super::*;

/// No lockfile at all is the ordinary state of a machine that never ran `shall lock`, and a
/// malformed one is not worth a second failure on top of the first. Both stay quiet.
#[test]
fn a_missing_or_unreadable_lockfile_is_silent_rather_than_a_second_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = Config {
        config_root: dir.path().to_path_buf(),
        ..Default::default()
    };
    assert!(on_install_failure(&config, "apt", "libudev1", APT_SAID).is_none());

    let (_kept, broken) = config_with("{ this is not json");
    assert!(on_install_failure(&broken, "apt", "libudev1", APT_SAID).is_none());
}
