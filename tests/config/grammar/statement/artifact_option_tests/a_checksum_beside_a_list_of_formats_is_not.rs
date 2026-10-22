use super::*;

#[test]
fn a_checksum_beside_a_list_of_formats_is_not() {
    let err = p("github:sharkdp/fd@sha256=abc,formats=deb,formats=rpm").unwrap_err();
    assert!(format!("{}", err).contains("lists 2"), "{}", err);
}
