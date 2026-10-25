use super::*;

#[test]
fn lvm_remove_confirms_and_names_the_path() {
    assert_eq!(lvm_remove("vg0", "data"), vec!["-y", "vg0/data"]);
}
