use super::*;

/// E18/E19: one condition had two message families and the busier one named the backend
/// rather than the program it probed, so `lvm` told you to install `lvm` while looking for
/// `lvs`. Both implementations are now one function, and it is told what was probed.
#[test]
fn a_missing_program_is_named_by_the_program_that_was_probed() {
    use shall::core::missing_program;

    let r = missing_program("lvm", &["lvs".to_string()]);
    assert_eq!(r.status, HealthStatus::Absent);
    let m = r.message.unwrap();
    assert!(m.contains("`lvs`"), "{m}");
    assert!(!m.contains("Binary for"), "the old message survived: {m}");

    // An absolute path is not "not on PATH" (U16).
    let m = missing_program("vendor", &["/opt/vendor/thing".to_string()])
        .message
        .unwrap();
    assert!(m.contains("does not exist or is not executable"), "{m}");

    // Two programs, and no claim about how many of them are needed.
    let m = missing_program("krew", &["kubectl".into(), "kubectl-krew".into()])
        .message
        .unwrap();
    assert!(
        m.contains("`kubectl`") && m.contains("`kubectl-krew`"),
        "{m}"
    );

    // A backend that probes nothing must not be described as missing a program.
    let m = missing_program("appimage", &[]).message.unwrap();
    assert!(!m.contains('`') || m.contains("`appimage`"), "{m}");
}
