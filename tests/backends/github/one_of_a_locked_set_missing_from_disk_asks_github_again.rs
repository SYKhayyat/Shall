use super::*;

#[test]
fn one_of_a_locked_set_missing_from_disk_asks_github_again() {
    let locked = [
        lock("v10.2.0", "fd.tar.gz", "tarball"),
        lock("v10.2.0", "fd-server.tar.gz", "tarball"),
    ];
    let state = installed("v10.2.0", &["fd.tar.gz"]);
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: Some(&state),
    };
    assert!(!answered_locally(
        &known,
        &tarballs(),
        Some(&AssetPattern::parse("all").unwrap())
    ));
}
