use super::*;

/// A `@reboot` job keeps its log redirection. It did not: the boot shape was written by
/// overwriting the file the ordinary shape had just written, and the replacement had no
/// `StandardOutput=` at all — so the one kind of job nobody watches run wrote its output
/// nowhere.
#[test]
fn a_boot_job_still_says_where_its_output_goes() {
    let unit =
        LinuxSystemdProvisioner::service_unit(&cfg("@reboot"), Path::new("/bin/shall"), &here());
    assert!(unit.contains("StandardOutput=append:"), "{}", unit);
    assert!(unit.contains("StandardError=append:"), "{}", unit);
    assert!(unit.contains("WantedBy=default.target"), "{}", unit);
    // And it has no timer to be installed into.
    let p = LinuxSystemdProvisioner;
    assert!(!p
        .rendered(&cfg("@reboot"), Path::new("/bin/shall"), &here())
        .unwrap()
        .spec
        .contains("OnCalendar="));
}
