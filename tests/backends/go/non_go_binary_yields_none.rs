use super::*;

#[test]
fn non_go_binary_yields_none() {
    assert!(parse_go_version_m("some random text\nno module info here\n").is_none());
}
