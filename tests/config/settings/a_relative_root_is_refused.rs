use super::*;

#[test]
fn a_relative_root_is_refused() {
    let err = Settings::parse("config_root = \"../shall\"", &at("settings.toml")).unwrap_err();
    assert!(err.to_string().contains("absolute"));
}
