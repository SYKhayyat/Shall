use super::*;

#[test]
fn rollup_of_nothing_is_empty() {
    assert!(backend_rollup(&[]).is_empty());
}
