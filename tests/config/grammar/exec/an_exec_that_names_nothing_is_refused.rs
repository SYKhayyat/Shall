use super::*;

#[test]
fn an_exec_that_names_nothing_is_refused() {
    assert!(p("exec:").is_err());
}
