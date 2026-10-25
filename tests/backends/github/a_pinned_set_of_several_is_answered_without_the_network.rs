use super::*;

#[test]
fn a_pinned_set_of_several_is_answered_without_the_network() {
    // `@asset=all` locks every file it installed; all of them present is the answer.
    let locked = [
        lock("v10.2.0", "fd.tar.gz", "tarball"),
        lock("v10.2.0", "fd-server.tar.gz", "tarball"),
    ];
    let state = installed("v10.2.0", &["fd-server.tar.gz", "fd.tar.gz"]);
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: Some(&state),
    };
    assert!(answered_locally(
        &known,
        &tarballs(),
        Some(&AssetPattern::parse("all").unwrap())
    ));
}
