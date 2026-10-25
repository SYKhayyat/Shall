use super::*;

#[test]
fn the_settings_file_is_never_inside_the_repo_it_locates() {
    // The default repo is `<config dir>/shall`. A settings file nested under it would be
    // committed to git and would carry one machine's absolute path to every other.
    let settings = Settings::path();
    let default_repo = shall::utils::safe_config_dir();
    assert!(
        !settings.starts_with(&default_repo),
        "{} is inside the default repo {}",
        settings.display(),
        default_repo.display()
    );
}
