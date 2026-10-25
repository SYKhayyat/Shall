use super::*;

#[test]
fn the_windows_spec_names_what_runs_when_and_at_which_privilege() {
    let mut elevated = cfg("0 3 * * *");
    elevated.elevated = Some(true);
    let bin = Path::new("C:\\Program Files\\shall\\shall.exe");
    let plain = WindowsTaskProvisioner
        .rendered(&cfg("0 3 * * *"), bin, &here())
        .unwrap();
    let raised = WindowsTaskProvisioner
        .rendered(&elevated, bin, &here())
        .unwrap();
    assert!(plain.spec.contains("LeastPrivilege"), "{}", plain.spec);
    assert!(raised.spec.contains("HighestAvailable"), "{}", raised.spec);
    assert!(plain.spec.contains("daily at 03:00"), "{}", plain.spec);
    assert!(
        plain
            .spec
            .contains("C:\\Program Files\\shall\\shall.exe sync"),
        "{}",
        plain.spec
    );
    assert_ne!(plain.spec, raised.spec);
}
