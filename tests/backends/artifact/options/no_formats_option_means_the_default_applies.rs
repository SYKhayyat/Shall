use super::*;

#[test]
fn no_formats_option_means_the_default_applies() {
    let read = ArtifactOptions::read(&opts(&[])).unwrap();
    let fallback = FormatOrder::new(vec![Format::Tarball]);
    assert_eq!(read.resolved_formats(&fallback), fallback);
}
