use super::*;

#[test]
fn a_pattern_narrows_but_does_not_select() {
    let formats = FormatOrder::new(vec![Format::Deb]);
    let platform = linux64();
    let pattern = AssetPattern::parse("*amd64*").unwrap();
    let found = select(
        &request(&formats, &platform, Some(&pattern)),
        &assets(&["fd-musl_10.2.0_amd64.deb", "fd_10.2.0_amd64.deb"]),
    )
    .unwrap();
    assert_eq!(found.picks.len(), 1, "still one pick, tie-broken as usual");
    assert_eq!(found.picks[0].asset.name, "fd_10.2.0_amd64.deb");
}
