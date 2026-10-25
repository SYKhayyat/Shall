use super::*;

#[test]
fn a_config_provider_missing_a_required_command_is_refused() {
    let mut d = def("lvm");
    d.create = vec![];
    assert!(d.unusable().is_some());
    let mut d = def("lvm");
    d.list = vec![];
    assert!(d.unusable().is_some());
    let mut d = def("lvm");
    d.list_pattern = String::new();
    assert!(d.unusable().is_some());
    assert!(def("lvm").unusable().is_none(), "a complete row is usable");
}
