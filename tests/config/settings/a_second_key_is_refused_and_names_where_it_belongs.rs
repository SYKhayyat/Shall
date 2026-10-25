use super::*;

#[test]
fn a_second_key_is_refused_and_names_where_it_belongs() {
    let err = Settings::parse(
        &format!(
            "config_root = \"{}\"\nverbose = true",
            absolute("srv/shall")
        ),
        &at("settings.toml"),
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("verbose"));
    assert!(
        msg.contains("preferences.toml"),
        "the refusal must say where the key does belong"
    );
}
