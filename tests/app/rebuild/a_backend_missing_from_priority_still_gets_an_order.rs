use super::*;

#[test]
fn a_backend_missing_from_priority_still_gets_an_order() {
    // Falling back to usize::MAX puts it last rather than dropping it; sorting by name
    // after that keeps the run reproducible.
    let backends = vec!["zig".to_string(), "npm".to_string(), "cargo".to_string()];
    let priority = vec!["cargo".to_string()];
    assert_eq!(
        order_backends(&backends, &priority, &foundation),
        vec!["cargo".to_string(), "npm".to_string(), "zig".to_string()]
    );
}
