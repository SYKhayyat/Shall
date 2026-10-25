use super::*;

/// U27/V.60: a config provider is create-only unless the file *names* the live-restore
/// capability. The default — omit the field — is the safe one, and even declaring the flag is
/// not enough without a `restore` command to run.
#[test]
fn a_config_provider_is_create_only_unless_it_declares_both() {
    let mut d = def("lvm");
    assert!(!d.is_live(), "the default must be create-only");

    d.restores_running_system = true;
    assert!(
        !d.is_live(),
        "the flag alone, with no restore command, is not live"
    );

    d.restore = vec!["merge".into(), "{id}".into()];
    assert!(d.is_live(), "flag AND a restore command is live");
}
