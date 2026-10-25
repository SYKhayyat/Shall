use super::*;

/// The table parses from `preferences.toml` under the name the docs give it, and a key that
/// is left out keeps its default rather than zeroing the struct.
#[test]
fn the_lock_table_parses_and_a_missing_key_keeps_its_default() {
    let config: Config = toml::from_str("[lock]\nreplay = false\n").expect("parse [lock]");
    assert!(!config.lock.replay, "the key that was given");
    assert_eq!(
        config.lock.versions,
        vec!["*".to_string()],
        "a key that was not given must keep its default, not empty out"
    );
    assert_eq!(config.lock.freeze, vec!["everything".to_string()]);
    assert_eq!(config.lock.freezes(), shall::core::lock_kind::ALL.to_vec());

    // `deny_unknown_fields` on the table, so a typo is a loud failure and not a setting
    // that reads as configured while being off — the same argument `Config` itself makes.
    assert!(
        toml::from_str::<Config>("[lock]\nreplayy = false\n").is_err(),
        "a misspelled key inside [lock] must be refused"
    );
}
