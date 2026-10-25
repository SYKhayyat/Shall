use super::*;

#[test]
fn a_setting_written_in_the_file_survives_a_flag_nobody_passed() {
    for (name, _set_flag, set_file, read) in flag_pairs() {
        let mut config = Config::default();
        set_file(&mut config);
        config.merge_cli_overrides(CliOverrides::default());
        assert!(
            read(&config),
            "`{name} = true` in preferences.toml was erased by a command line that never \
             mentioned {name}. There is no `--no-{}` flag, so an absent flag cannot mean off.",
            name.replace('_', "-")
        );
    }
}
