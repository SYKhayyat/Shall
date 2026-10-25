use super::*;

#[test]
fn two_accounts_get_two_distinct_unit_paths() {
    let account = shall::core::account::Account::current().unwrap();
    let theirs = me();
    let mine = TaskTarget::resolve(None).unwrap();
    assert_eq!(
        LinuxSystemdProvisioner::unit_dir(&mine).unwrap(),
        dirs::config_dir().unwrap().join("systemd").join("user")
    );
    assert_eq!(
        LinuxSystemdProvisioner::unit_dir(&theirs).unwrap(),
        account.systemd_user_dir().unwrap()
    );
}
