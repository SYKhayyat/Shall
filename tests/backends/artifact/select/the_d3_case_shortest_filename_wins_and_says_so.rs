use super::*;

#[test]
fn the_d3_case_shortest_filename_wins_and_says_so() {
    let formats = FormatOrder::new(vec![Format::Deb]);
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&["fd-musl_10.2.0_amd64.deb", "fd_10.2.0_amd64.deb"]),
    )
    .unwrap();
    assert_eq!(found.picks[0].asset.name, "fd_10.2.0_amd64.deb");
    assert!(found.was_ambiguous());
    assert_eq!(found.passed_over[0].name, "fd-musl_10.2.0_amd64.deb");
}
