use super::*;

#[test]
fn a_pinned_installed_package_is_answered_without_the_network() {
    let locked = [lock("v10.2.0", "fd.tar.gz", "tarball")];
    let state = installed("v10.2.0", &["fd.tar.gz"]);
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: Some(&state),
    };
    assert!(answered_locally(&known, &tarballs(), None));
}
