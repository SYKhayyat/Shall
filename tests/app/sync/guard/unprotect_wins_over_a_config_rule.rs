use super::*;

#[test]
fn unprotect_wins_over_a_config_rule() {
    let mut cfg = config_with(20);
    cfg.guard.unprotected_packages = vec!["libpam-modules".into()];
    let none = HashSet::new();
    // libpam* still protects the rest of the family...
    assert!(protection_of(&cfg, Some("apt"), "libpam0g", &none).is_some());
    // ...but the explicit opt-out wins for the one the user named.
    assert!(protection_of(&cfg, Some("apt"), "libpam-modules", &none).is_none());
}
