use super::*;

#[test]
fn an_extensionless_asset_has_no_format() {
    assert_eq!(Format::of_filename("fd-linux"), None);
    assert_eq!(Format::of_filename("fd"), None);
}
