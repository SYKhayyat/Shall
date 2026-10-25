use super::*;

#[test]
fn an_unscoped_schedule_uses_what_it_always_used() {
    let _env = shall::core::shall_data_dir_lock();
    let here = TaskTarget::resolve(None).unwrap();
    assert!(!here.is_scoped());
    assert_eq!(
        LinuxSystemdProvisioner::unit_dir(&here).unwrap(),
        dirs::config_dir().unwrap().join("systemd").join("user")
    );
    assert_eq!(
        here.log_path(),
        shall::utils::safe_data_dir().join("schedule.log"),
        "an unscoped schedule writes where it has always written, whatever the platform's \
         data directory is"
    );
    assert!(here.account().is_ok());
}
