use super::*;

#[test]
fn one_package_gets_the_per_package_budget() {
    assert_eq!(node_deadline(NODE, 1, TOTAL), NODE);
}
