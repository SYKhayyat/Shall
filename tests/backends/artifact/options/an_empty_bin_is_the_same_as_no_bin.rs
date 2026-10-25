use super::*;

#[test]
fn an_empty_bin_is_the_same_as_no_bin() {
    let read = ArtifactOptions::read(&opts(&[("bin", "   ")])).unwrap();
    assert!(read.bin.is_none());
}
