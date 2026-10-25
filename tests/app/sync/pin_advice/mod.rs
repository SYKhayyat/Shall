pub use shall::app::sync::pin_advice::*;
pub use shall::config::Config;

fn config_with(pins: &str) -> (tempfile::TempDir, Config) {
    let dir = tempfile::tempdir().expect("tempdir");
    let locks = dir.path().join("locks");
    std::fs::create_dir_all(&locks).expect("locks dir");
    std::fs::write(locks.join("versions.json"), pins).expect("write pins");
    let config = Config {
        config_root: dir.path().to_path_buf(),
        ..Default::default()
    };
    (dir, config)
}
const APT_SAID: &str = "E: Version '255.4-1ubuntu8.17' for 'libudev1' was not found";
mod a_failure_on_the_recorded_version_says_where_the_version_came_from;
mod a_failure_that_does_not_quote_the_pin_gets_no_advice;
mod a_missing_or_unreadable_lockfile_is_silent_rather_than_a_second_failure;
mod a_package_that_is_not_pinned_is_never_blamed_on_a_pin;
