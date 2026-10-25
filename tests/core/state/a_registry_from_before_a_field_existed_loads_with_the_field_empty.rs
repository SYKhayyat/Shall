use super::*;

#[test]
fn a_registry_from_before_a_field_existed_loads_with_the_field_empty() {
    // Additive fields are schema evolution, not an old format: a registry with no
    // `suspensions` key was written by a build where suspension did not exist, so
    // *provably* nothing is suspended — filling it with empty is not a guess. (The
    // reverse case, a file from a NEWER build carrying keys this one lacks, is silently
    // ignored by serde either way; there was never a reader-side guard for that.)
    let missing = r#"{"packages":[],"active_session_id":null}"#;
    let r: StateRegistry = serde_json::from_str(missing).expect("additive absence loads");
    assert!(r.suspensions.is_empty());
    assert!(r.held.is_empty());
}
