use super::*;

#[test]
fn a_bare_name_still_carries_no_options() {
    assert!(parse("apt\ncargo\n").unwrap().options("apt").is_none());
}
