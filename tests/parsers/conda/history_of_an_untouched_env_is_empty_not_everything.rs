use super::*;

#[test]
fn history_of_an_untouched_env_is_empty_not_everything() {
    // The failure that matters: if this ever returned the full closure instead, `adopt`
    // would adopt an entire dependency graph.
    assert!(parse_conda_history(r#"{"name":"base","dependencies":[]}"#)
        .expect("an env nobody asked anything of has an empty `dependencies`")
        .is_empty());
}
