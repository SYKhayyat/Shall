use super::*;

#[test]
fn an_empty_guard_table_objects_to_nothing() {
    let guard = shall::config::GuardSettings::default();
    assert!(guard.is_empty());
    assert!(inspect_desired(&guard, &desired(&[("apt", "curl", None)])).is_empty());
}
