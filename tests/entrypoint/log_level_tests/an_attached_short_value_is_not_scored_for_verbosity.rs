use super::*;

/// An attached VALUE is not a bundle of flags: `-c$HOME/.prefs.toml` names the config
/// file, and the v's inside its path said nothing about verbosity — two of them turned
/// one quiet config into debug logging.
#[test]
fn an_attached_short_value_is_not_scored_for_verbosity() {
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-c/srv/vault/prefs.toml", "list"])),
        "warn"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-c/etc/shall.conf", "-v", "list"])),
        "info"
    );
}
