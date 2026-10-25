use super::*;

#[test]
fn an_empty_settings_file_is_valid_and_means_the_default() {
    let s = Settings::parse("", &at("settings.toml")).unwrap();
    assert!(s.config_root.is_none());
}
