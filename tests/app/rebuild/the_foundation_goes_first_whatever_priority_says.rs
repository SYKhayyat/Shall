use super::*;

#[test]
fn the_foundation_goes_first_whatever_priority_says() {
    let backends = vec!["cargo".to_string(), "apt".to_string()];
    let priority = vec!["cargo".to_string(), "apt".to_string()];
    assert_eq!(
        order_backends(&backends, &priority, &foundation),
        vec!["apt".to_string(), "cargo".to_string()],
        "a crate can need a system compiler; no apt package needs a crate"
    );
}
