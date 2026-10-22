use super::*;

#[test]
fn an_absolute_expiry_is_accepted() {
    assert!(p("apt:jq@expires=2026-07-17T14:00").is_ok());
    assert!(p("apt:jq@expires=2026-07-17T14:00:00Z").is_ok());
}
