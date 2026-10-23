/// `envs()` adds to what the child inherits, so a value Shall sets must win over the same
/// name in the environment it was started with.
/// The fabricated statuses used to be `/bin/false` and `cmd /C exit 1` with `.expect()` on
/// the spawn — a panic on the one path whose premise is that nothing runs.
#[test]
fn a_fabricated_status_carries_the_code_without_spawning_anything() {
    let ok: std::process::Output = super::DryRunOutput::new().into();
    assert!(ok.status.success());
    assert_eq!(ok.status.code(), Some(0));

    let bad = super::DryRunOutput::faulted("E: no package index");
    assert!(!bad.status.success());
    assert_eq!(bad.status.code(), Some(1));
    assert_eq!(bad.stderr, b"E: no package index");
}
