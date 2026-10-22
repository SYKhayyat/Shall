use super::*;

#[test]
fn asset_all_and_a_checksum_cannot_both_be_set() {
    let err = p("github:sharkdp/fd@asset=all,sha256=abc").unwrap_err();
    assert!(format!("{}", err).contains("cannot both be set"));
}
