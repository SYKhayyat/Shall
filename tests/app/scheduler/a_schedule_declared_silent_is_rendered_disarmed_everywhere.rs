use super::*;

/// `enabled = false` is a declaration about the machine, not a comment: it changes what
/// `rendered` claims, so a task somebody armed by hand reports as drift.
#[test]
fn a_schedule_declared_silent_is_rendered_disarmed_everywhere() {
    let mut silent = cfg("0 2 * * *");
    silent.enabled = Some(false);
    let bin = Path::new("/bin/shall");
    assert!(
        !LinuxSystemdProvisioner
            .rendered(&silent, bin, &here())
            .unwrap()
            .armed
    );
    assert!(
        !MacLaunchdProvisioner
            .rendered(&silent, bin, &here())
            .unwrap()
            .armed
    );
    assert!(
        !WindowsTaskProvisioner
            .rendered(&silent, bin, &here())
            .unwrap()
            .armed
    );
    // launchd carries it in the file as well, so a reboot does not arm it behind us.
    assert!(MacLaunchdProvisioner
        .plist(&silent, bin, &here())
        .contains("<key>Disabled</key><true/>"));
    assert!(!MacLaunchdProvisioner
        .plist(&cfg("0 2 * * *"), bin, &here())
        .contains("Disabled"));
}
