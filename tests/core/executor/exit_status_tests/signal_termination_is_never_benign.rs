use super::*;

#[test]
fn signal_termination_is_never_benign() {
    let choco = executor_for(exit_policy::choco());
    let killed = StdOutput {
        status: fabricate_status(0),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    // `fabricate_status` always yields a code, so exercise the None arm directly.
    assert!(!exit_policy::choco().is_benign(None));
    assert!(!exit_policy::winget().is_benign(None));
    assert!(choco.ensure_status("choco", killed).is_ok());
}
