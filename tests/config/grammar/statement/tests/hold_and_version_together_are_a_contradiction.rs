use super::*;

#[test]
fn hold_and_version_together_are_a_contradiction() {
    let err = p("apt:jq@hold,version=1.6").unwrap_err();
    assert!(err.what.contains("contradict"), "{}", err.what);
}
