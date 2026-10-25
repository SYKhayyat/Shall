use super::*;

#[test]
fn your_repo_holds_what_you_wrote() {
    let l = layout();
    assert_eq!(l.modules_dir(), PathBuf::from("/cfg/modules"));
    assert_eq!(l.profiles_dir(), PathBuf::from("/cfg/profiles"));
    assert_eq!(l.active_file(), PathBuf::from("/cfg/active"));
    assert_eq!(l.priority_file(), PathBuf::from("/cfg/priority"));
    assert_eq!(l.schedules_file(), PathBuf::from("/cfg/schedules"));
    assert_eq!(l.locks_dir(), PathBuf::from("/cfg/locks"));
    assert_eq!(l.preferences_file(), PathBuf::from("/cfg/preferences.toml"));
}
