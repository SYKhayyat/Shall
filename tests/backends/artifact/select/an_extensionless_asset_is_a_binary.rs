use super::*;

#[test]
fn an_extensionless_asset_is_a_binary() {
    let formats = FormatOrder::new(vec![Format::Binary]);
    let platform = linux64();
    let found = select(&request(&formats, &platform, None), &assets(&["fd-linux"])).unwrap();
    assert_eq!(found.picks[0].format, Format::Binary);
}
