use super::*;

#[test]
fn a_pin_that_moved_past_the_lock_asks_github() {
    let locked = [lock("v10.2.0", "fd.tar.gz", "tarball")];
    let state = installed("v10.2.0", &["fd.tar.gz"]);
    let known = Known {
        pin: Some("10.3.0"),
        locked: &locked,
        installed: Some(&state),
    };
    assert!(!answered_locally(&known, &tarballs(), None));
}
