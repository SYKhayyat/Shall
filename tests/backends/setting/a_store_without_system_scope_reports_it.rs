use super::*;

/// A store with no machine-wide commands says so rather than quietly writing the per-user
/// value — the silent wrong answer P7 refuses.
#[test]
fn a_store_without_system_scope_reports_it() {
    assert!(!gsettings().has_system_scope());
    // ...and asking for system scope falls back to nothing: the caller must refuse, which
    // `scope_of` does. The argv itself never silently becomes the user one for a caller
    // that checked first.
    let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
    let err = core
        .scope_of(&gsettings(), Some("system"), "org.gnome.x/k", None)
        .expect_err("system scope on a user-only store must be refused")
        .to_string();
    assert!(err.contains("gsettings"), "{}", err);
    assert!(err.contains("one account"), "{}", err);
}
