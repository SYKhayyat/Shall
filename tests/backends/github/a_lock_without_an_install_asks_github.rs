use super::*;

#[test]
fn a_lock_without_an_install_asks_github() {
    let locked = [lock("v10.2.0", "fd.tar.gz", "tarball")];
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: None,
    };
    assert!(!answered_locally(&known, &tarballs(), None));
}
