use super::*;

#[test]
fn detection_is_case_insensitive() {
    assert_eq!(Format::of_filename("Tool.AppImage"), Some(Format::AppImage));
    assert_eq!(Format::of_filename("TOOL.DEB"), Some(Format::Deb));
}
