use super::*;

#[test]
fn lvm_name_splits_into_group_and_volume() {
    assert_eq!(split_lvm("vg0/data").unwrap(), ("vg0", "data"));
    // A name that is not `group/volume` is refused, not guessed at.
    assert!(split_lvm("data").is_err());
    assert!(split_lvm("vg0/").is_err());
    assert!(split_lvm("/data").is_err());
}
