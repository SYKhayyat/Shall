use super::*;

#[test]
fn a_release_with_no_assets_says_so_rather_than_listing_nothing() {
    let formats = FormatOrder::new(vec![Format::Deb]);
    let platform = linux64();
    let err = select(&request(&formats, &platform, None), &[]).unwrap_err();
    assert!(err.to_string().contains("no assets at all"));
}
