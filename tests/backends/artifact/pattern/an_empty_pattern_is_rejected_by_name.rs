use super::*;

#[test]
fn an_empty_pattern_is_rejected_by_name() {
    let err = AssetPattern::parse("  ").unwrap_err();
    assert!(err.to_string().contains("all"));
}
