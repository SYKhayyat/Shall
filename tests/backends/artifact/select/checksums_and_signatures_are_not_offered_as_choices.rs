use super::*;

#[test]
fn checksums_and_signatures_are_not_offered_as_choices() {
    let formats = FormatOrder::new(vec![Format::Deb]);
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&[
            "fd_10.2.0_amd64.deb",
            "fd_10.2.0_amd64.deb.sha256",
            "checksums.txt",
        ]),
    )
    .unwrap();
    assert_eq!(found.picks[0].asset.name, "fd_10.2.0_amd64.deb");
    assert!(!found.was_ambiguous());
}
