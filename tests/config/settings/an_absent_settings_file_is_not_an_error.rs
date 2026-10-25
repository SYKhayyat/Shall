use super::*;

#[test]
fn an_absent_settings_file_is_not_an_error() {
    let s = Settings::load_from(&at("definitely/not/here/settings.toml")).unwrap();
    assert!(s.config_root.is_none());
}
