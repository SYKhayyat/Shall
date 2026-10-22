use super::*;

#[test]
fn there_is_no_fallback_across_channels() {
    let err = p("snap:code@channel=edge,channel=stable").unwrap_err();
    assert!(format!("{}", err).contains("one value"));
}
