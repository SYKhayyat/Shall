use super::*;

#[test]
fn nothing_may_follow_list_in_a_chain() {
    // Unreachable syntax that parses is syntax that lies about what the line does.
    let err = p("list,apt:ripgrep").unwrap_err();
    assert!(err.what.contains("must come last"), "{}", err.what);
}
