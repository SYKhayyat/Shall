use super::*;

/// The ruling: an ordinary run prints its answer and nothing else.
#[test]
fn an_ordinary_run_says_nothing_about_itself() {
    assert_eq!(log_level_from_argv(&argv(&["shall", "list"])), "warn");
    assert_eq!(log_level_from_argv(&argv(&["shall", "sync"])), "warn");
}
