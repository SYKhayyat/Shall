use super::*;

#[test]
fn a_drifted_host_reports_its_counts() {
    let r = parse_check(DRIFTED).unwrap();
    assert_eq!((r.to_install, r.to_remove, r.unmanaged), (2, 1, 3));
    assert!(r.drifted);
}
