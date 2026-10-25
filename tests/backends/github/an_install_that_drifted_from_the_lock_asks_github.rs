use super::*;

#[test]
fn an_install_that_drifted_from_the_lock_asks_github() {
    let locked = [lock("v10.2.0", "fd-gnu.tar.gz", "tarball")];
    let state = installed("v10.2.0", &["fd-musl.tar.gz"]);
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: Some(&state),
    };
    assert!(!answered_locally(&known, &tarballs(), None));
}
