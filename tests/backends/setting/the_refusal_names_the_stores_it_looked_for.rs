use super::*;

/// The refusal names what Shall looked for, so a machine running an unlisted store learns
/// what to write a row about rather than only that it failed.
#[test]
fn the_refusal_names_the_stores_it_looked_for() {
    let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
    let msg = core
        .no_adapter("org.gnome.desktop.interface/color-scheme")
        .to_string();
    assert!(msg.contains("gsettings"), "{}", msg);
    assert!(msg.contains("setting_store"), "{}", msg);
}
