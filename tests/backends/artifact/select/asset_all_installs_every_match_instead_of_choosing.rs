use super::*;

#[test]
fn asset_all_installs_every_match_instead_of_choosing() {
    let formats = FormatOrder::new(vec![Format::Deb, Format::Tarball]);
    let platform = linux64();
    let pattern = AssetPattern::parse("all").unwrap();
    let found = select(
        &request(&formats, &platform, Some(&pattern)),
        &assets(&["fd_10.2.0_amd64.deb", "fd-linux-x86_64.tar.gz"]),
    )
    .unwrap();
    assert_eq!(found.picks.len(), 2);
    assert!(!found.was_ambiguous(), "nothing was passed over");
}
