use super::*;

#[test]
fn a_setting_nobody_asked_for_stays_off() {
    // The other half of the ratchet: `|=` must not be a way for a default to drift on.
    let mut config = Config::default();
    config.merge_cli_overrides(CliOverrides::default());
    for (name, _, _, read) in flag_pairs() {
        assert!(!read(&config), "{name} turned itself on");
    }
}
