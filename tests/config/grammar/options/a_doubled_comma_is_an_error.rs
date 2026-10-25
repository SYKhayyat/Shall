use super::*;

#[test]
fn a_doubled_comma_is_an_error() {
    assert!(parse_short(&o(), "hold,,version=1").is_err());
    assert!(parse_short(&o(), "hold,").is_err());
}
