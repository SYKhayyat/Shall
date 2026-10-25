use super::*;

#[test]
fn a_later_format_is_a_fallback_not_an_addition() {
    let formats = FormatOrder::new(vec![Format::Deb, Format::Tarball]);
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&["fd-linux-x86_64.tar.gz"]),
    )
    .unwrap();
    assert_eq!(found.picks.len(), 1);
    assert_eq!(found.picks[0].format, Format::Tarball);
}
