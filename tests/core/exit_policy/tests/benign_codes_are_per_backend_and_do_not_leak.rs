use super::*;

#[test]
fn benign_codes_are_per_backend_and_do_not_leak() {
    for code in [1605, 1614, 1618, 1641, 3010] {
        assert!(choco().is_benign(Some(code)));
        assert!(!winget().is_benign(Some(code)));
        assert!(!apt().is_benign(Some(code)));
    }
    assert!(!choco().is_benign(Some(1)));
    assert!(winget().is_benign(Some(-1978335189)));
    assert!(winget().is_benign(Some(-1978335212)));
    assert!(!winget().is_benign(Some(1)));
    assert!(!apt().is_benign(Some(100)));
    assert!(!apk().is_benign(Some(1)));
    assert!(!dnf().is_benign(Some(1)));
}
