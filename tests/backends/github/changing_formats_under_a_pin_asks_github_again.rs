use super::*;

#[test]
fn changing_formats_under_a_pin_asks_github_again() {
    let locked = [lock("v10.2.0", "fd.tar.gz", "tarball")];
    let state = installed("v10.2.0", &["fd.tar.gz"]);
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: Some(&state),
    };
    assert!(!answered_locally(
        &known,
        &FormatOrder::new(vec![Format::Deb]),
        None
    ));
}
