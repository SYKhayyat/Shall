use super::*;

#[test]
fn hash_changes_with_real_options_and_removals() {
    let base = vec![spec("apt", "htop", None)];
    let versioned = vec![spec("apt", "htop", Some(("version", "3.0")))];
    assert_ne!(
        hash_plan(&base, &[], &PlanResources::default()),
        hash_plan(&versioned, &[], &PlanResources::default())
    );

    let with_removal = hash_plan(
        &base,
        &[PlanRemoval {
            backend: "apt".into(),
            name: "vim".into(),
        }],
        &PlanResources::default(),
    );
    assert_ne!(
        hash_plan(&base, &[], &PlanResources::default()),
        with_removal
    );
}
