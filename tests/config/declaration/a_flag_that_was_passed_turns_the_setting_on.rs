use super::*;

#[test]
fn a_flag_that_was_passed_turns_the_setting_on() {
    for (name, set_flag, _set_file, read) in flag_pairs() {
        let mut overrides = CliOverrides::default();
        set_flag(&mut overrides);
        let mut config = Config::default();
        config.merge_cli_overrides(overrides);
        assert!(read(&config), "--{} did not reach the config", name);
    }
}
