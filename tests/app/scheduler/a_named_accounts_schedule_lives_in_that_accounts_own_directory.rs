use super::*;

#[test]
fn a_named_accounts_schedule_lives_in_that_accounts_own_directory() {
    let account = shall::core::account::Account::current().unwrap();
    let target = me();
    assert!(target.is_scoped());
    assert_eq!(target.account().unwrap(), account);

    let dir = LinuxSystemdProvisioner::unit_dir(&target).unwrap();
    assert_eq!(dir, account.systemd_user_dir().unwrap());
    assert!(
        dir.starts_with(account.home()),
        "{dir:?} is not in the account's home"
    );

    let log = target.log_path();
    assert!(
        log.starts_with(account.home()),
        "{log:?} is not in the account's home"
    );
    assert!(LinuxSystemdProvisioner::service_unit(
        &cfg("0 2 * * *"),
        Path::new("/bin/shall"),
        &target
    )
    .contains(&log.display().to_string()));
}
