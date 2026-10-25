use super::*;

#[test]
fn an_unpinned_line_always_asks_github() {
    let locked = [lock("v10.2.0", "fd.tar.gz", "tarball")];
    let state = installed("v10.2.0", &["fd.tar.gz"]);
    let known = Known {
        pin: None,
        locked: &locked,
        installed: Some(&state),
    };
    assert!(!answered_locally(&known, &tarballs(), None));
}
