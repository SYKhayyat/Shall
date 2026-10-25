use super::*;

#[test]
fn a_user_written_format_order_wins_over_a_more_specific_asset() {
    // `@formats=deb;tarball` is an instruction, so the deb wins even though the tarball
    // names the machine more precisely (D2). This is the half where format order leads.
    let formats = FormatOrder::new(vec![Format::Deb, Format::Tarball]).as_user_specified();
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&["fd-linux-x86_64.tar.gz", "fd_10.2.0_amd64.deb"]),
    )
    .unwrap();
    assert_eq!(found.picks[0].format, Format::Deb);
    assert_eq!(found.picks[0].asset.name, "fd_10.2.0_amd64.deb");
}
