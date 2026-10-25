use super::*;

#[test]
fn a_foreign_asset_is_never_chosen_even_as_a_last_resort() {
    let formats = FormatOrder::new(vec![Format::Zip]);
    let platform = linux64();
    let err = select(
        &request(&formats, &platform, None),
        &assets(&["fd-x86_64-pc-windows-msvc.zip"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not for this machine"));
}
