use super::*;

/// `lvm:` was unusable by construction: `lvcreate` has no default size, so the backend
/// refused every line without `@size` and the parser refused every line with one. The
/// backend's own error told the user to write a line the grammar rejected.
#[test]
fn a_volume_can_be_given_the_size_lvm_requires() {
    assert_eq!(opt("lvm:vg0/data@size=10G", "size"), "10G");
    assert_eq!(opt("lvm:vg0/data@size=64M", "size"), "64M");
}
