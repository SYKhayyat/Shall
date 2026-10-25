use super::*;

#[test]
fn a_single_format_needs_no_separator() {
    let read = ArtifactOptions::read(&opts(&[("formats", "deb")])).unwrap();
    assert_eq!(read.formats.unwrap().as_slice(), &[Format::Deb]);
}
