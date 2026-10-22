use super::*;

#[test]
fn a_checksum_needs_the_line_to_pin_one_format() {
    // D6: `github:x/y@sha256=…` with no format pinned means the Debian box downloads the
    // `.deb` and the Fedora box the `.rpm`, and one hash cannot verify two files.
    let err = p("github:sharkdp/fd@sha256=abc").unwrap_err();
    assert!(format!("{}", err).contains("exactly one format"), "{}", err);
    assert!(format!("{}", err).contains("locks/"), "{}", err);
}
