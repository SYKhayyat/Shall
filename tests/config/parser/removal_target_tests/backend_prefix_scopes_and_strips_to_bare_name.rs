use super::*;

#[test]
fn backend_prefix_scopes_and_strips_to_bare_name() {
    assert_eq!(
        split_removal_target("uv:ruff", known),
        (Some("uv".to_string()), "ruff".to_string())
    );
    assert_eq!(
        split_removal_target("apt:tree", known),
        (Some("apt".to_string()), "tree".to_string())
    );
}
