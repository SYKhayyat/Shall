use super::*;

#[test]
fn lvm_create_needs_a_group_size_and_volume() {
    assert_eq!(
        lvm_create("vg0", "data", "10G"),
        vec!["-n", "data", "-L", "10G", "vg0"]
    );
}
