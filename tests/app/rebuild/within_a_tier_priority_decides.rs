use super::*;

#[test]
fn within_a_tier_priority_decides() {
    let backends = vec!["npm".to_string(), "cargo".to_string()];
    let priority = vec!["cargo".to_string(), "npm".to_string()];
    assert_eq!(
        order_backends(&backends, &priority, &foundation),
        vec!["cargo".to_string(), "npm".to_string()]
    );
}
