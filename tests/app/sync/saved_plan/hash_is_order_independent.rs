use super::*;

#[test]
fn hash_is_order_independent() {
    let a = vec![spec("apt", "htop", None), spec("cargo", "ripgrep", None)];
    let b = vec![spec("cargo", "ripgrep", None), spec("apt", "htop", None)];
    assert_eq!(
        hash_plan(&a, &[], &PlanResources::default()),
        hash_plan(&b, &[], &PlanResources::default())
    );
}
