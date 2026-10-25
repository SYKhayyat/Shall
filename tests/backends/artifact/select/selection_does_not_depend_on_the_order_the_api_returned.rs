use super::*;

#[test]
fn selection_does_not_depend_on_the_order_the_api_returned() {
    let formats = FormatOrder::new(vec![Format::Deb]);
    let platform = linux64();
    let forwards = select(
        &request(&formats, &platform, None),
        &assets(&["fd-a_amd64.deb", "fd-b_amd64.deb"]),
    )
    .unwrap();
    let backwards = select(
        &request(&formats, &platform, None),
        &assets(&["fd-b_amd64.deb", "fd-a_amd64.deb"]),
    )
    .unwrap();
    assert_eq!(forwards.picks, backwards.picks);
}
