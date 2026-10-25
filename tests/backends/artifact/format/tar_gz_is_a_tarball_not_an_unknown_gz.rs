use super::*;

#[test]
fn tar_gz_is_a_tarball_not_an_unknown_gz() {
    assert_eq!(
        Format::of_filename("fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz"),
        Some(Format::Tarball)
    );
    assert_eq!(Format::of_filename("x.tgz"), Some(Format::Tarball));
    assert_eq!(Format::of_filename("x.tar.zst"), Some(Format::Tarball));
}
