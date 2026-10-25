use super::*;

#[test]
fn a_converged_host_reports_in_sync() {
    let r = parse_check(CONVERGED).unwrap();
    assert_eq!((r.to_install, r.to_remove, r.unmanaged), (0, 0, 0));
    assert!(!r.drifted);
}
