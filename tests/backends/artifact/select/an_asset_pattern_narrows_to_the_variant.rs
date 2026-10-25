use super::*;

#[test]
fn an_asset_pattern_narrows_to_the_variant() {
    let formats = FormatOrder::new(vec![Format::Deb]);
    let platform = linux64();
    let pattern = AssetPattern::parse("*musl*").unwrap();
    let found = select(
        &request(&formats, &platform, Some(&pattern)),
        &assets(&["fd-musl_10.2.0_amd64.deb", "fd_10.2.0_amd64.deb"]),
    )
    .unwrap();
    assert_eq!(found.picks[0].asset.name, "fd-musl_10.2.0_amd64.deb");
    assert!(!found.was_ambiguous());
}
