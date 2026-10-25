use super::*;

#[test]
fn a_detected_order_yields_to_the_asset_that_names_the_machine() {
    // Same two assets, but the order was DETECTED, not written. Now the tarball wins
    // because it names `linux-x86_64` while the deb names only `amd64` — the jq case.
    let formats = FormatOrder::new(vec![Format::Deb, Format::Tarball]);
    assert!(!formats.is_user_specified());
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&["fd-linux-x86_64.tar.gz", "fd_10.2.0_amd64.deb"]),
    )
    .unwrap();
    assert_eq!(found.picks[0].asset.name, "fd-linux-x86_64.tar.gz");
}
