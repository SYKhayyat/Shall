use super::*;

#[test]
fn an_asset_pattern_is_validated_at_parse_time() {
    assert_eq!(
        options_of("github:sharkdp/fd@asset=*musl*").one("asset"),
        Some("*musl*")
    );
}
