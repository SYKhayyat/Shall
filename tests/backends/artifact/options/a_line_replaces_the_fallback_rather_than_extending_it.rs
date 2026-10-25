use super::*;

#[test]
fn a_line_replaces_the_fallback_rather_than_extending_it() {
    let read = ArtifactOptions::read(&opts(&[("formats", "rpm")])).unwrap();
    let fallback = FormatOrder::new(vec![Format::Deb, Format::Tarball]);
    assert_eq!(read.resolved_formats(&fallback).as_slice(), &[Format::Rpm]);
}
