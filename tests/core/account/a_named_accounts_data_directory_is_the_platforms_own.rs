use super::*;

/// **Every platform's convention, asserted from any host.** The macOS leg of this is the one
/// that was wrong, and a `#[cfg(target_os = "macos")]` test would have caught it only on the
/// machine that reported it.
#[test]
fn a_named_accounts_data_directory_is_the_platforms_own() {
    let home = Path::new("/home/somebody");
    assert_eq!(
        data_dir_under(home),
        match host_platform() {
            Platform::MacOs => home.join("Library/Application Support/shall"),
            Platform::Unix => home.join(".local/share/shall"),
            Platform::Windows => home.join("AppData/Local/shall"),
        }
    );
    // The two conventions that differ, named rather than left to the host's cfg.
    assert_eq!(
        data_dir_under(home),
        if cfg!(target_os = "macos") {
            home.join("Library/Application Support/shall")
        } else if cfg!(windows) {
            home.join("AppData/Local/shall")
        } else {
            home.join(".local/share/shall")
        },
        "an account's data directory is not the platform's data directory convention"
    );
}
