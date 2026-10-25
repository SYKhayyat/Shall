use super::*;

#[test]
fn config_template_actually_parses_and_matches_the_defaults() {
    // `shall config init` writes this file verbatim. A template that does not parse
    // hands every new user a broken config, and a template whose keys don't match the
    // struct silently documents settings that do nothing (as `cache_ttl` did).
    let cfg: shall::config::Config =
        toml::from_str(CONFIG_TEMPLATE).expect("CONFIG_TEMPLATE must be valid preferences.toml");
    assert_eq!(cfg.guard.max_removals, 20);
}
