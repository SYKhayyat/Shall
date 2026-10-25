use super::*;

#[test]
fn an_artifact_nobody_owns_hands_nothing_back() {
    let d = Deployed::default()
        .owned(None, Some("ripgrep"))
        .owned(Some("apt"), None)
        .owned(Some("apt"), Some("ripgrep"));
    assert_eq!(d.owned_by, vec![("apt".to_string(), "ripgrep".to_string())]);
}
