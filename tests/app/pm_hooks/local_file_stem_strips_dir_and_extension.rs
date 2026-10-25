use super::*;

#[test]
fn local_file_stem_strips_dir_and_extension() {
    assert_eq!(local_file_stem("/tmp/htop_3.0_amd64.deb"), "htop_3.0_amd64");
    assert_eq!(local_file_stem("foo-1.0.pkg.tar.zst"), "foo-1.0");
    assert_eq!(local_file_stem("bar.rpm"), "bar");
    assert_eq!(local_file_stem("plainname"), "plainname");
}
