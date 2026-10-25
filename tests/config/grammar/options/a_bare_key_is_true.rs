use super::*;

#[test]
fn a_bare_key_is_true() {
    let opts = parse_short(&o(), "hold").unwrap();
    assert_eq!(opts.one("hold"), Some("true"));
}
