use super::*;

#[test]
fn a_repeated_key_keeps_the_order_it_was_written_in() {
    let read = ArtifactOptions::read(&opts(&[("formats", "deb;tarball;binary")])).unwrap();
    assert_eq!(
        read.formats.unwrap().as_slice(),
        &[Format::Deb, Format::Tarball, Format::Binary]
    );
}
