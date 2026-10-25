use super::*;

#[test]
fn a_signal_kill_is_never_benign() {
    assert!(!choco().is_benign(None));
    assert!(!winget().is_benign(None));
}
