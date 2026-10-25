use super::*;

#[test]
fn a_version_number_in_the_name_is_not_an_extension() {
    let formats = FormatOrder::new(vec![Format::Binary]);
    let platform = linux64();
    let found = select(
        &request(&formats, &platform, None),
        &assets(&["fd-v10.2.0-x86_64-linux"]),
    )
    .unwrap();
    assert_eq!(found.picks[0].asset.name, "fd-v10.2.0-x86_64-linux");
}
