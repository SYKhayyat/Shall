use super::*;

#[test]
fn a_download_with_no_checksum_is_refused() {
    assert!(check_checksum_declared(&spec(&[])).is_err());
    assert!(check_checksum_declared(&spec(&[("sha256", "abc")])).is_ok());
    assert!(check_checksum_declared(&spec(&[("unverified", "true")])).is_ok());
}
