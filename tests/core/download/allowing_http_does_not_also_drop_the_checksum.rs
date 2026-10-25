use super::*;

#[test]
fn allowing_http_does_not_also_drop_the_checksum() {
    // The whole point of keeping them separate: over HTTP the checksum is the only thing
    // left, so the flag that permits HTTP must not be the flag that removes it.
    let s = spec(&[("allow_http", "true")]);
    assert!(allows_http(&s));
    assert!(check_checksum_declared(&s).is_err());
}
