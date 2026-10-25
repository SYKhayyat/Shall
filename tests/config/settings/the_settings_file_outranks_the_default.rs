use super::*;

#[test]
fn the_settings_file_outranks_the_default() {
    let stored = at(&absolute("stored"));
    let settings = Settings {
        config_root: Some(stored.clone()),
    };
    let resolved = resolve_root(None, &settings).unwrap();
    assert_eq!(resolved.path, stored);
    assert_eq!(resolved.source, RootSource::SettingsFile);
}
