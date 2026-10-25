use super::*;

#[test]
fn hash_ignores_internal_provenance_options() {
    let a = vec![spec("apt", "htop", Some(("__source", "local.txt")))];
    let b = vec![spec("apt", "htop", Some(("__source", "module:dev")))];
    assert_eq!(
        hash_plan(&a, &[], &PlanResources::default()),
        hash_plan(&b, &[], &PlanResources::default())
    );
}
